use std::fmt;
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use lazy_regex::regex_captures;
use lazy_static::lazy_static;
use std::collections::HashMap;
use geomorph::Utm;
use utm::to_utm_wgs84;
use std::ops;
use std::str::FromStr;

#[derive(Copy, Debug, Clone)]
pub struct Coord3 {
    pub e: f32,
    pub n: f32,
    pub h: f32,
}

lazy_static! {
    static ref LOCATIONS: HashMap<&'static str, &'static str> = HashMap::from([
	("Austerdalsbreen", "N6857378.59E74028.82"),
	("Bukkehåmåren", "N6831287.57E165104.69"),
	("Dalegubben", "N6929342.17E55699.65"),
	("Dørålseter", "N6884975.42E228065.39"),
	("Galdhøpiggen", "N6851889.09E146005.17"),
	("Giklingdalen", "N6968433.83E181437.49"),
	("Gråkallen", "N7041229.73E263033.76"),
	("Higravtind", "N7582614.25E491443.74"),
	("Innerdalen", "N6970663.77E181965.81"),
	("Jønshornet", "N6939567.47E51789.75"),
	("Koven", "N7801561.74E796000.84"),
	("Kufot", "N7777944.37E829160.64"),
	("Litjdalen", "N6957527.09E167573.23"),
	("Litlefjellet", "N6951428.83E129294.17"),
	("Lodalskåpa", "N6875511.46E89605.11"),
	("Loenvatnet", "N6878404.9E78921.26"),
	("Neådalssnota", "N6975732.57E196332.68"),
	("Nordre Sætertind", "N6934326.09E52020.75"),
	("Nordre Trolltind", "N6949920.69E125714.78"),
	("Olsanestinden", "N7590523.96E503865.44"),
	("Midtronden", "N6878653.14E230391.25"),
	("Olstinden", "N7539262.19E419471.91"),
	("Rødøyløva", "N7396875.03E413808.27"),
	("Sanna", "N7379422.66E368557.76"),
	("Sautso", "N7761024.88E838717.86"),
	("Slogen", "N6925227.33E67695.5"),
	("Smedhamran", "N6877556.88E225420.61"),
	("Smørstabbtindan", "N6844576.5E135670.28"),
	("Snøheim", "N6919748.71E207190.05"),
	("Snøhetta", "N6922988.3E203182.98"),
	("Stetinden", "N7562126.7E566097.85"),
	("Store Knutholstind", "N6827003.55E156852.26"),
	("Store Ringstind", "N6833238.42E116579.44"),
	("Store Skagastølstind", "N6834962.93E120609"),
	("Store Vengetind", "N6951177.34E131787.15"),
	("Storsylen", "N6990928.53E358250.73"),
	("Torghatten", "N7255964.08E364892.09"),
    ]);
}

impl Coord3 {
    pub fn new(e: f32, n: f32, h:f32) -> Coord3 {
	Coord3 { e: e, n: n, h: h }
    }

    pub fn dot(&self, other: Coord3) -> f32 {
	self.e*other.e + self.n*other.n + self.h*other.h
    }

    // Rotate around vertial axis
    pub fn rot_h(&self, angle: f32) -> Coord3 {
	Coord3 { e: self.e*angle.cos() - self.n*angle.sin(),
		 n: self.e*angle.sin() + self.n*angle.cos(),
		 h: self.h
	}
    }

    // Rotate around east axis
    pub fn rot_e(&self, angle: f32) -> Coord3 {
	Coord3 { e: self.e,
		 n: self.n*angle.cos() - self.h*angle.sin(),
		 h: self.n*angle.sin() + self.h*angle.cos()
	}
    }

    // Absolute length from origo
    pub fn abs(&self) -> f32 {
	(self.e*self.e + self.n*self.n + self.h*self.h).sqrt()
    }
}

impl fmt::Display for Coord3 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
	formatter.write_fmt(format_args!("({}, {}, {})",
					 self.e, self.n, self.h))
    }
}

#[derive(Copy, Debug, Clone, PartialEq)]
pub struct Coord {
    pub e: f32,
    pub n: f32,
}

impl Coord {
    pub fn new(e: f32, n: f32) -> Coord {
	Coord { e: e, n: n }
    }

    pub fn from_polar(r: f32, phi: f32) -> Coord {
	Coord { e: r*phi.cos(), n: r*phi.sin() }
    }

    pub fn from_latlon(lat: f64, lon: f64) -> Self {
        /*
        let gc = geomorph::Coord::new(lat, lon);
        let utm = Utm::from(gc);
        */

        let (n, e, _) = to_utm_wgs84(lat, lon, 33);
        // FIXME: Are we guaranteed to get the right north (true), band (W)
        // and ups (false) for our norwegian coordinates?
        Self {
            e: e as f32,
            n: n as f32,
        }
    }
    
    // Absolute length from origo
    pub fn abs(&self) -> f32 {
	(self.e*self.e + self.n*self.n).sqrt()
    }

    pub fn abs_sq(&self) -> f32 {
	self.e*self.e + self.n*self.n
    }

    pub fn dot(&self, o: &Coord) -> f32 {
        self.e*o.e + self.n*o.n
    }

    pub fn rot90(&self) -> Coord {
        Self {
            e: - self.n,
            n: self.e,
        }
    }

    pub fn normalize(&self) -> Coord {
        let abs = self.abs();
        Self {
            e: self.e/abs,
            n: self.n/abs,
        }
    }

    pub fn is_finite(&self) -> bool {
        return self.e.is_finite() && self.n.is_finite();
    }

    // Return longitude and latitude
    pub fn latlon(&self) -> (f64, f64) {
        let utm = Utm::new(
            self.e as f64, self.n as f64, true, 33, 'W', false);
        let gc : geomorph::Coord = utm.into();
        return (gc.lat, gc.lon);
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
	formatter.write_fmt(format_args!("N{}E{}", self.n, self.e))
    }
}

impl From<&str> for Coord {
    fn from(s: &str) -> Self {
        if let Ok(c) = s.parse() {
            return c;
        }
        else {
	    panic!("Invalid coordinate {}", s);
        }
    }
}

impl FromStr for Coord {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
	if LOCATIONS.contains_key(s) {
	    return Ok(Coord::from(*LOCATIONS.get(s).unwrap()));
	}

	let res = regex_captures!("N(-?[0-9.]+)E(-?[0-9.]+)$", s);
	if let Some((_, n, e)) = res {
	    Ok(Coord { e: e.parse().unwrap(), n: n.parse().unwrap() })
	}
	else {
            Err(format!("Invalid coordinate {}", s))
	}
    }
}

impl ops::Add<Coord> for Coord {
    type Output = Coord;

    fn add(self, _rhs: Coord) -> Coord {
	Coord { e: self.e + _rhs.e, n: self.n + _rhs.n }
    }
}

impl ops::Sub<Coord> for Coord {
    type Output = Coord;

    fn sub(self, _rhs: Coord) -> Coord {
	Coord { e: self.e - _rhs.e, n: self.n - _rhs.n }
    }
}

impl ops::Mul<f32> for Coord {
    type Output = Coord;

    fn mul(self, _rhs: f32) -> Coord {
	Coord { e: self.e*_rhs, n: self.n*_rhs }
    }
}

impl ops::AddAssign<Coord> for Coord {
    fn add_assign(&mut self, rhs: Coord) {
        self.e += rhs.e;
        self.n += rhs.n;
    }
}

impl Serialize for Coord {
    fn serialize<S>(&self, serializer: S)
                    -> std::result::Result<S::Ok, S::Error>
    where S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Coord {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Coord, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Coord::from(s.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use crate::coord::Coord;

    #[test]
    fn new_coord() {
	let c = Coord::new(10.5, -11.3);
	assert_eq!(c.e, 10.5);
	assert_eq!(c.n, -11.3);
    }

    #[test]
    fn format() {
	let c = Coord::new(10.5, -11.3);
	let f = format!("{}", c);
	assert_eq!(f, "N-11.3E10.5");
    }
}
