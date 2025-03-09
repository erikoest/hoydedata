use crate::errors::*;
use crate::coord::Coord;
use crate::mapfolder::{ZipMount, map_dir};
use crate::atlas::MsgSender;

extern crate exif;
use exif::{Exif, Tag, In, Context, Value};
use gdal::{Dataset};
use std::collections::HashSet;
use std::cell::RefCell;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub fname: String,
    pub zipfile: String,
    width: usize,
    height: usize,
    pub nw: Coord,
    pub se: Coord,
    pub delta: Coord,
    #[serde(skip_serializing, skip_deserializing)]
    im: RefCell<Vec<f32>>,
}

impl Map {
    fn exif_u32(exif: &Exif, tag: Tag) -> Option<u32> {
	if let Some(field) = exif.get_field(tag, In::PRIMARY) {
	    return field.value.get_uint(0);
	}

	None
    }

    fn exif_float(exif: &Exif, tag: Tag, i: usize) -> Option<f32> {
	if let Some(field) = exif.get_field(tag, In::PRIMARY) {
	    return match field.value {
		Value::Double(ref v) if v.len() > i => Some(v[i] as f32),
		_ => None,
	    }
	}

	None
    }

    pub fn new(fname: &str, zipfile: &str, tx: Option<&MsgSender>)
               -> Result<Self> {
	let absfile = format!("{}{}", map_dir(), fname);
	let file = std::fs::File::open(absfile).unwrap();
	let mut bufreader = std::io::BufReader::new(&file);
	let exifreader = exif::Reader::new();
	let exif = exifreader.read_from_container(&mut bufreader).unwrap();

	let width = Self::exif_u32(&exif, Tag::ImageWidth).unwrap() as usize;
	let height = Self::exif_u32(&exif, Tag::ImageLength).unwrap() as usize;

	let delta = Coord::new(
	    Self::exif_float(&exif, Tag(Context::Tiff, 0x830E), 0).unwrap(),
	    Self::exif_float(&exif, Tag(Context::Tiff, 0x830E), 1).unwrap()
	);

	let nw = Coord::new(
	    Self::exif_float(&exif, Tag(Context::Tiff, 0x8482),
			     3).unwrap(),
	    Self::exif_float(&exif, Tag(Context::Tiff, 0x8482),
			     4).unwrap()
	);
	
	let se = nw + Coord::new(
            (width as f32)*delta.e,
            - (height as f32)*delta.n
        );

        if let Some(some_tx) = tx {
            some_tx.send(format!("Map: {} {} -> {}", fname, nw, se)).unwrap();
        }

	Ok(Self {
	    fname: String::from(fname),
	    zipfile: String::from(zipfile),
	    width: width,
	    height: height,
	    nw: nw,
	    se: se,
	    delta: delta,
	    im: Default::default(),
	})
    }

    pub fn resolution(&self) -> f32 {
	self.delta.n
    }
    
    pub fn coord_to_hash(coord: &Coord) -> i32 {
        let h = (((coord.n - 6400000.0)/500.0) as i32)*10000 +
	    (((coord.e + 120000.0)/500.0) as i32);
        h
    }

    pub fn hashes(&self) -> HashSet<i32> {
        let mut ret = HashSet::new();

        let mut y = self.se.n;
        while y <= self.nw.n {
            let mut x = self.nw.e;
	    while x <= self.se.e {
                ret.insert(Map::coord_to_hash(&Coord::new(x, y)));
                x += 500.0;
	    }
	    y += 500.0;
	}

        ret
    }

    pub fn is_loaded(&self) -> bool {
        self.im != Default::default()
    }
    
    pub fn load_image(&self, tx: Option<&MsgSender>) -> Result<()> {
        if let Some(some_tx) = tx {
            some_tx.send(format!("Reading file {}", self.fname)).unwrap();
        }

	if self.zipfile != "" {
	    let _ = ZipMount::new(&self.zipfile);
	}

        let absname = format!("{}{}", map_dir(), self.fname);
	let im = Dataset::open(absname)?;
	let band = &im.rasterband(1);

	// Copy the whole raster array into an f32 vector
	let window_size = (self.width, self.height);
	let size = (self.width, self.height);
	let resample_alg = None;
	let window = (0, 0);
	let rv = band.as_ref().unwrap().read_as::<f32>(window, window_size,
                                                       size, resample_alg)?;
        self.im.replace(rv.data);
	Ok(())
    }

    pub fn lookup(&self, coord: &Coord) -> Result<f32> {
        /*
        Lookup height for coordinate. Function will load complete height
	data if not already loaded.
	 */
        let x = ((coord.e - self.nw.e)/self.delta.e) as isize;
        let y = ((self.nw.n - coord.n)/self.delta.n) as isize;

	// We require the point to be one sample from the map edge. We want the
	// lookup function to have the same restrictions as the
	// lookup_with_gradient function.
        if x < 1 || x >= (self.width as isize) -1 ||
	    y < 1 || y >= (self.height as isize) -1 {
		return Err(Error::LookupError(
                    coord.clone(),
                    String::from(&self.fname)).into()
                );
	    }

        if !self.is_loaded() {
	    return Err(Error::MapNotLoaded(String::from(&self.fname)).into());
	}

	Ok(self.im.borrow()[x as usize + (y as usize)*self.width])
    }

    /*
    Lookup height of coordinate. Also return gradient deduced from neighbouring
    points. The return value is the triple (height, dh/dx, dh/dy)
     */
    pub fn lookup_with_gradient(&self, coord: &Coord)
                                -> Result<(f32, f32, f32)> {
        /*
        Lookup height for coordinate. Function will load complete height
	data if not already loaded.
	 */
        let x = ((coord.e - self.nw.e)/self.delta.e) as isize;
        let y = ((self.nw.n - coord.n)/self.delta.n) as isize;

	// We require the point to be one sample from the map edge in order for
	// us to calculate the gradient.
        if x < 1 || x >= (self.width as isize) - 1 ||
	    y < 1 || y >= (self.height as isize) - 1 {
		return Err(Error::LookupError(
                    coord.clone(),
                    String::from(&self.fname)).into()
                );
	    }

        if !self.is_loaded() {
	    return Err(Error::MapNotLoaded(String::from(&self.fname)).into());
	}

	let i = x as usize + (y as usize)*self.width;
	let a = self.im.borrow();
        let h = a[i];
	let dx_1 = h - a[i - 1];
	let dx_2 = a[i + 1] - h;
	let dy_1 = a[i - (self.width as usize)] - h;
	let dy_2 = h - a[i + (self.width as usize)];

	Ok((h, (dx_1 + dx_2)*0.5/self.delta.e, (dy_1 + dy_2)*0.5/self.delta.n))
    }
}

#[cfg(test)]
mod tests {
    use crate::map::Map;
    use crate::coord::*;
    use std::collections::HashSet;

    #[test]
    fn new_from_fname() {
	let _ = Map::new("testdata/6700_4_10m_z33.tif", "", None);
    }

    #[test]
    fn coord_to_hash() {
	assert_eq!(Map::coord_to_hash(&Coord::new(100.0, 6789745.0)), 7790240);
    }

    #[test]
    fn hashes() {
	let m = Map::new("testdata/6700_4_10m_z33.tif", "", None).unwrap();
	let h = m.hashes();
	let s = HashSet::from(
            [7730294, 7140317, 7580295, 7600324, 7930241, 7340239, 7900279,
	     7530327, 7270323, 7500290, 7910264, 7360310, 7430290, 7260294,
	     7290321, 7540290, 7060258, 7670242, 7380283, 7420269, 7140321]
        );
	// panic!("Hashes returned: {}", h.iter().map( |id| id.to_string() + ",").collect::<String>());
	assert!(s.is_subset(&h));
    }

    #[test]
    fn load_image() {
	let m = Map::new("testdata/6700_4_10m_z33.tif", "", None).unwrap();
	match m.load_image(None) {
	    Ok(r) => assert_eq!(r, ()),
	    Err(err) => panic!("{}", err),
	}
    }
    
    #[test]
    fn lookup() {
	let m = Map::new("testdata/6700_4_10m_z33.tif", "", None).unwrap();
	match m.lookup(&Coord::new(100.0, 6789745.0)) {
	    Ok(r)    => assert_eq!(645.61273, r),
	    Err(err) => panic!("{}", err),
	}
    }

    #[test]
    fn lookup_failure() {
	let m = Map::new("testdata/6700_4_10m_z33.tif", "", None).unwrap();
	match m.lookup(&Coord::new(-100000.0, 6789745.0)) {
	    Ok(r)    => assert_eq!(645.61273, r),
	    Err(err) => assert_eq!(
                err.to_string(),
                "Lookup 'N6789745E-100000' on map 'testdata/6700_4_10m_z33.tif' failed"
            ),
	}
    }

    #[test]
    fn lookup_with_gradient() {
	let m = Map::new("testdata/6700_4_10m_z33.tif", "", None).unwrap();
	match m.lookup_with_gradient(&Coord::new(100.0, 6789745.0)) {
	    Ok(r)    => assert_eq!((645.61273, 0.20289306, -0.6372711), r),
	    Err(err) => panic!("{}", err),
	}
    }
}
