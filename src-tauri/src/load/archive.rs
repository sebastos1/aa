use anyhow::{Context, Result};
use walkdir::WalkDir;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum FileCategory {
    Advancement,
    Tags,
    Recipe,
    Language,
    // Textures
}

impl FileCategory {
    pub fn matches(&self, path: &str) -> bool {
        if path.contains("/datapacks/") { return false }
        
        match self {
            FileCategory::Advancement => {
                path.contains("/advancement/") && path.ends_with(".json") && !path.contains("/recipes/")
            }
            FileCategory::Tags => {
                path.contains("/tags/") && path.ends_with(".json")
            }
            FileCategory::Recipe => {
                path.contains("/recipe/") && path.ends_with(".json")
            }
            FileCategory::Language => {
                path.ends_with("/lang/en_us.json")
            }
        }
    }
}

pub trait Archive {
    fn list_files(&mut self, categories: &[FileCategory]) -> Result<Vec<String>>;
    fn read_file(&mut self, path: &str) -> Result<String>;
    fn name(&self) -> &str;
}

/// zip and jar
pub struct ZipArchive {
    zip: zip::ZipArchive<std::fs::File>,
    name: String,
}

impl ZipArchive {
    pub fn new(path: &Path, name: String) -> Result<Self> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("Failed to open archive: {}", path.display()))?;
        let zip = zip::ZipArchive::new(file)
            .with_context(|| format!("Failed to read archive: {}", path.display()))?;
        
        Ok(Self { zip, name })
    }
}

impl Archive for ZipArchive {
    fn list_files(&mut self, categories: &[FileCategory]) -> Result<Vec<String>> {
        let mut relevant_files = Vec::new();
        
        for i in 0..self.zip.len() {
            if let Ok(file) = self.zip.by_index(i) {
                let file_path = file.name();
                
                for category in categories {
                    if category.matches(file_path) {
                        relevant_files.push(file_path.to_string());
                        break;
                    }
                }
            }
        }
        
        Ok(relevant_files)
    }
    
    fn read_file(&mut self, path: &str) -> Result<String> {
        let mut file = self.zip.by_name(path).with_context(|| format!("File not found in archive: {}", path))?;
        let mut content = String::new();
        file.read_to_string(&mut content).with_context(|| format!("Failed to read file from archive: {}", path))?;
        Ok(content)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// regular dirs
pub struct DirArchive {
    root: PathBuf,
    name: String,
}

impl DirArchive {
    pub fn new(root: PathBuf, name: String) -> Self {
        Self { root, name }
    }
}

impl Archive for DirArchive {
    fn list_files(&mut self, categories: &[FileCategory]) -> Result<Vec<String>> {
        let mut relevant_files = Vec::new();
        
        if !self.root.exists() {
            return Ok(relevant_files);
        }
        
        let mut dirs = HashSet::new();
        
        for category in categories {
            match category {
                FileCategory::Advancement | FileCategory::Tags | FileCategory::Recipe => {
                    dirs.insert("data".to_string());
                }
                FileCategory::Language => {
                    dirs.insert("assets".to_string());
                }
            }
        }
        
        for search_dir in dirs.into_iter() {
            let full_search_path = self.root.join(&search_dir);
            if !full_search_path.exists() {
                continue;
            }
            
            for entry in WalkDir::new(&full_search_path) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let relative = entry.path().strip_prefix(&self.root)
                        .with_context(|| format!("Failed to get relative path for: {}", entry.path().display()))?;
                    let file_path = relative.to_string_lossy().replace('\\', "/");
                    
                    // Check if this file matches any of our categories
                    for category in categories {
                        if category.matches(&file_path) {
                            relevant_files.push(file_path);
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(relevant_files)
    }
    
    fn read_file(&mut self, path: &str) -> Result<String> {
        let full_path = self.root.join(path);
        fs::read_to_string(&full_path)
            .with_context(|| format!("Failed to read file: {}", full_path.display()))
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

pub fn archive_from_jar_path(jar_path: &Path) -> Result<Box<dyn Archive>> {
    ZipArchive::new(jar_path, "minecraft".to_string()).map(|zip| Box::new(zip) as Box<dyn Archive>)
}

// factory
pub fn open_archive(path: &Path, name: String) -> Result<Box<dyn Archive>> {
    if path.is_dir() {
        Ok(Box::new(DirArchive::new(path.to_path_buf(), name)))
    } else if path.extension() == Some(std::ffi::OsStr::new("zip")) || path.extension() == Some(std::ffi::OsStr::new("jar")) {
        Ok(Box::new(ZipArchive::new(path, name)?))
    } else {
        Err(anyhow::anyhow!("Could not open {}", path.display()))
    }
}

// get advancement id from path
pub fn extract_advancement_id(file_path: &str) -> String {
    let parts: Vec<&str> = file_path.split('/').collect();
    if parts.len() >= 4 && parts[0] == "data" && parts[2] == "advancement" {
        let namespace = parts[1];
        let advancement_path = parts[3..]
            .join("/")
            .strip_suffix(".json")
            .unwrap_or("")
            .to_string();
        
        if namespace == "minecraft" {
            advancement_path
        } else {
            format!("{}:{}", namespace, advancement_path)
        }
    } else {
        file_path.strip_suffix(".json").unwrap_or(file_path).to_string()
    }
}