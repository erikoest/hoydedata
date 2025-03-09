extern crate hoydedata;

use hoydedata::{Atlas, Coord, Result, set_map_dir, unmount_all_maps};

use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    set_map_dir("/media/ekstern/hoydedata/");
    
    let a = Atlas::new(10.0, None)?;

    let c = Coord::from(args[1].as_str());
    println!("Coordinate is {}", c);
    
    for m in a.lookup_maps(&c)? {
	println!("Map: {}", m.fname);
    }

    let height = a.lookup(&c)?;
    println!("Height: {}", height);
    
    unmount_all_maps();
    
    Ok(())
}
