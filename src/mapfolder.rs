use std::fs;
use std::process::Command;

use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref MAPFOLDER: MapFolder = MapFolder::new();
}

// FIXME: Create an Atlas::flush_maps() method which also unmounts the
// map directories
pub fn unmount_all_maps() {
    MAPFOLDER.unmount_all()
}

pub fn set_map_dir(dir: &str) {
    MAPFOLDER.set_dir(dir)
}

pub fn map_dir() -> String {
    MAPFOLDER.get_map_dir()
}

pub struct MapFolder {
    mounts: Mutex<HashMap<String, bool>>,
    map_dir: Mutex<Option<String>>,
}

impl MapFolder {
    pub fn new() -> Self {
	Self {
	    mounts: Mutex::new(HashMap::new()),
            map_dir: Mutex::new(None),
	}
    }

    pub fn set_dir(&self, dir: &str) {
        self.map_dir.lock().unwrap().replace(dir.to_string());
    }

    pub fn get_map_dir(&self) -> String {
        let guard = self.map_dir.lock().unwrap();
        if let Some(d) = guard.as_ref() {
            return d.clone();
        }
        else {
            panic!("Map dir is not loaded");
        }
    }

    pub fn is_mounted(&self, filename: &str) -> bool {
	self.mounts.lock().unwrap().contains_key(filename)
    }
    
    pub fn register(&self, filename: &str) {
	self.mounts.lock().unwrap().insert(String::from(filename), true);
    }

    pub fn unmount_all(&self) {
	// Unmount all registered mount directories
	for k in self.mounts.lock().unwrap().keys() {
	    let absdir = format!("{}{}.dir", map_dir(), &k);

	    Command::new("/usr/bin/fusermount")
		.arg("-u")
		.arg(&absdir)
		.output()
		.expect("failed to execute process");

	    // Remove directory
	    fs::remove_dir(&absdir).unwrap();
	}

	self.mounts.lock().unwrap().clear();
    }
}

pub struct ZipMount {
    pub directory: String,
}

impl ZipMount {
    pub fn new(filename: &str) -> Self {
	// Check that file is zip file
	if !(filename.ends_with(".zip")) {
	    panic!("No zip file");
	}

	let directory = format!("{}.dir/", filename);
	let absdir = format!("{}{}.dir", map_dir(), filename);

	if !MAPFOLDER.is_mounted(filename) {
	    // Only mount first time.
	    // Create directory <zipfile>.dir
	    fs::create_dir_all(&absdir).unwrap();

	    // fuse-zip -r <zipfile> <zipfile>.dir
	    Command::new("/usr/bin/fuse-zip")
		.arg("-r")
		.arg(format!("{}{}", map_dir(), &filename))
		.arg(absdir)
		.output()
		.expect("failed to execute process");

	    MAPFOLDER.register(filename);
	}

	Self {
	    directory: directory.clone(),
	}
    }
}
