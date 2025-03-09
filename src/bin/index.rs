use hoydedata::{unmount_all_maps, Atlas, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut dir = args[2].to_string();
    if !dir.ends_with('/') {
        dir.push_str("/");
    }

    let file = &args[3];
    let afile;
    let a;

    if file == "" {
	// No file. Index directory.
	a = Atlas::new_from_directory("", "", None)?;
	afile = format!("{}{}", dir, "atlas.json");
    }
    else {
	a = Atlas::new_from_zip_file(&file, None)?;
	afile = format!("{}{}{}", dir, file, ".atlas.json");
    }

    a.write_atlas(&afile)?;

    unmount_all_maps();

    Ok(())
}
