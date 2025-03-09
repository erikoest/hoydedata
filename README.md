# hoydedata

Library for looking up elevation level from geotiff files. The library was made
specially for being used with the norwegian geotiff maps available from
https://hoydedata.no. It has not been tested on other data sets, and will probably
have a few limitations to that. Especially, the coordinate system is currently
limited to EU89 UTM33.

The module can work directly on tiff files and on zipfiles containing a set of tiff
files. In the latter case, the zip file is mounted as a virtual file system as
needed (using fusermount).

The lookup functionality is organized as atlases of maps, one atlas for each
map resolution level. The atlas contains indexes to each of its maps. The indexes
are stored on files at the location of the zip files, and they must be created
using the `index` tool (see next section).

Maps are loaded into memory only when needed. Map loading is begin done during
the lookup function. When the atlas object is constructed, only the index is
loaded into memory.

## Usage

  * Build library with helper tools:
  <pre>
    cargo build --release
  </pre>

  * Download geotiff maps from https://hoydedata.no/LaserInnsyn2
    (select 'eksport' -> 'landsdekkende')

  * Index maps in each zip file:
  <pre>
    cd /my/geodata/maps
    for z in *.zip; do
      (...)/hoydedata/target/release/index --maps . z
    done
  </pre>

  This creates index files, one for each of the zip files. If the maps are not
  zipped, an index file can be created for each of the individual tiff files as
  well.

  * Use the atlas lookup function in your code
  <pre>
    use hoydedata::{Atlas, Coord, set_map_dir, unmount_all_maps};

    set_map_dir("/media/ekstern/hoydedata/");

    let a = Atlas::new(10.0, None)?;
    let c = Coord::from("N6851889.09E146005.17");
    println!("Height level at {}: {}", c, a.lookup_maps(&c));

    unmount_all_maps();
  </pre>

## Utilities

### Index

Used for creating atlas-files from a zipped package of geotiff maps,
or a directory of geotiff files.

<pre>
index --maps &lt;mapdir&gt; &lt;zipfile&gt;
index --maps &lt;mapdir&gt; &lt;tiffdir&gt;
</pre>

### Lookup

Demo application using the atlas lookup function.
