//! Lib loader - loads built-in types based on target and lib options
//!
//! This module provides a loader that creates the correct set of built-in
//! type declarations based on the compiler's target ECMAScript version
//! and any additional lib options specified.

use super::types::LibDeclarations;
use super::{es5, es2015, es2020, dom};

/// ECMAScript version targets
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScriptTarget {
    ES3,
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2023,
    ESNext,
    JSON,
    Latest,
}

impl Default for ScriptTarget {
    fn default() -> Self {
        ScriptTarget::ES5
    }
}

impl ScriptTarget {
    /// Parse a target string to ScriptTarget
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "es3" => Some(ScriptTarget::ES3),
            "es5" => Some(ScriptTarget::ES5),
            "es6" | "es2015" => Some(ScriptTarget::ES2015),
            "es2016" => Some(ScriptTarget::ES2016),
            "es2017" => Some(ScriptTarget::ES2017),
            "es2018" => Some(ScriptTarget::ES2018),
            "es2019" => Some(ScriptTarget::ES2019),
            "es2020" => Some(ScriptTarget::ES2020),
            "es2021" => Some(ScriptTarget::ES2021),
            "es2022" => Some(ScriptTarget::ES2022),
            "es2023" => Some(ScriptTarget::ES2023),
            "esnext" => Some(ScriptTarget::ESNext),
            "json" => Some(ScriptTarget::JSON),
            "latest" => Some(ScriptTarget::Latest),
            _ => None,
        }
    }

    /// Get the target name as a string
    pub fn to_string(&self) -> &'static str {
        match self {
            ScriptTarget::ES3 => "ES3",
            ScriptTarget::ES5 => "ES5",
            ScriptTarget::ES2015 => "ES2015",
            ScriptTarget::ES2016 => "ES2016",
            ScriptTarget::ES2017 => "ES2017",
            ScriptTarget::ES2018 => "ES2018",
            ScriptTarget::ES2019 => "ES2019",
            ScriptTarget::ES2020 => "ES2020",
            ScriptTarget::ES2021 => "ES2021",
            ScriptTarget::ES2022 => "ES2022",
            ScriptTarget::ES2023 => "ES2023",
            ScriptTarget::ESNext => "ESNext",
            ScriptTarget::JSON => "JSON",
            ScriptTarget::Latest => "Latest",
        }
    }
}

/// Available lib files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LibFile {
    // Core ECMAScript libs
    ES5,
    ES2015,
    ES2015Core,
    ES2015Collection,
    ES2015Symbol,
    ES2015Iterable,
    ES2015Generator,
    ES2015Promise,
    ES2015Proxy,
    ES2015Reflect,
    ES2015SymbolWellKnown,
    ES2016,
    ES2016ArrayInclude,
    ES2017,
    ES2017Object,
    ES2017SharedMemory,
    ES2017String,
    ES2017Intl,
    ES2017TypedArrays,
    ES2018,
    ES2018AsyncGenerator,
    ES2018AsyncIterable,
    ES2018Intl,
    ES2018Promise,
    ES2018Regexp,
    ES2019,
    ES2019Array,
    ES2019Object,
    ES2019String,
    ES2019Symbol,
    ES2020,
    ES2020BigInt,
    ES2020Intl,
    ES2020Promise,
    ES2020SharedMemory,
    ES2020String,
    ES2020SymbolWellKnown,
    ES2021,
    ES2021Intl,
    ES2021Promise,
    ES2021String,
    ES2021WeakRef,
    ES2022,
    ES2022Array,
    ES2022Error,
    ES2022Intl,
    ES2022Object,
    ES2022SharedMemory,
    ES2022String,
    ES2022Regexp,
    ES2023,
    ES2023Array,
    ESNext,

    // DOM libs
    DOM,
    DOMIterable,
    WebWorker,
    WebWorkerImportScripts,
    ScriptHost,
    DOMAsyncIterable,

    // Decorators
    Decorators,
    DecoratorsLegacy,
}

impl LibFile {
    /// Parse a lib file name to LibFile
    pub fn from_str(s: &str) -> Option<Self> {
        let lower = s.to_lowercase();
        // Strip .d.ts suffix if present
        let name = lower.strip_suffix(".d.ts").unwrap_or(&lower);
        let name = name.strip_prefix("lib.").unwrap_or(name);

        match name {
            "es5" => Some(LibFile::ES5),
            "es2015" | "es6" => Some(LibFile::ES2015),
            "es2015.core" => Some(LibFile::ES2015Core),
            "es2015.collection" => Some(LibFile::ES2015Collection),
            "es2015.symbol" => Some(LibFile::ES2015Symbol),
            "es2015.iterable" => Some(LibFile::ES2015Iterable),
            "es2015.generator" => Some(LibFile::ES2015Generator),
            "es2015.promise" => Some(LibFile::ES2015Promise),
            "es2015.proxy" => Some(LibFile::ES2015Proxy),
            "es2015.reflect" => Some(LibFile::ES2015Reflect),
            "es2015.symbol.wellknown" => Some(LibFile::ES2015SymbolWellKnown),
            "es2016" => Some(LibFile::ES2016),
            "es2016.array.include" => Some(LibFile::ES2016ArrayInclude),
            "es2017" => Some(LibFile::ES2017),
            "es2017.object" => Some(LibFile::ES2017Object),
            "es2017.sharedmemory" => Some(LibFile::ES2017SharedMemory),
            "es2017.string" => Some(LibFile::ES2017String),
            "es2017.intl" => Some(LibFile::ES2017Intl),
            "es2017.typedarrays" => Some(LibFile::ES2017TypedArrays),
            "es2018" => Some(LibFile::ES2018),
            "es2018.asyncgenerator" | "es2018.asynciterable" => Some(LibFile::ES2018AsyncGenerator),
            "es2018.intl" => Some(LibFile::ES2018Intl),
            "es2018.promise" => Some(LibFile::ES2018Promise),
            "es2018.regexp" => Some(LibFile::ES2018Regexp),
            "es2019" => Some(LibFile::ES2019),
            "es2019.array" => Some(LibFile::ES2019Array),
            "es2019.object" => Some(LibFile::ES2019Object),
            "es2019.string" => Some(LibFile::ES2019String),
            "es2019.symbol" => Some(LibFile::ES2019Symbol),
            "es2020" => Some(LibFile::ES2020),
            "es2020.bigint" => Some(LibFile::ES2020BigInt),
            "es2020.intl" => Some(LibFile::ES2020Intl),
            "es2020.promise" => Some(LibFile::ES2020Promise),
            "es2020.sharedmemory" => Some(LibFile::ES2020SharedMemory),
            "es2020.string" => Some(LibFile::ES2020String),
            "es2020.symbol.wellknown" => Some(LibFile::ES2020SymbolWellKnown),
            "es2021" => Some(LibFile::ES2021),
            "es2021.intl" => Some(LibFile::ES2021Intl),
            "es2021.promise" => Some(LibFile::ES2021Promise),
            "es2021.string" => Some(LibFile::ES2021String),
            "es2021.weakref" => Some(LibFile::ES2021WeakRef),
            "es2022" => Some(LibFile::ES2022),
            "es2022.array" => Some(LibFile::ES2022Array),
            "es2022.error" => Some(LibFile::ES2022Error),
            "es2022.intl" => Some(LibFile::ES2022Intl),
            "es2022.object" => Some(LibFile::ES2022Object),
            "es2022.sharedmemory" => Some(LibFile::ES2022SharedMemory),
            "es2022.string" => Some(LibFile::ES2022String),
            "es2022.regexp" => Some(LibFile::ES2022Regexp),
            "es2023" => Some(LibFile::ES2023),
            "es2023.array" => Some(LibFile::ES2023Array),
            "esnext" => Some(LibFile::ESNext),
            "dom" => Some(LibFile::DOM),
            "dom.iterable" => Some(LibFile::DOMIterable),
            "dom.asynciterable" => Some(LibFile::DOMAsyncIterable),
            "webworker" => Some(LibFile::WebWorker),
            "webworker.importscripts" => Some(LibFile::WebWorkerImportScripts),
            "scripthost" => Some(LibFile::ScriptHost),
            "decorators" => Some(LibFile::Decorators),
            "decorators.legacy" => Some(LibFile::DecoratorsLegacy),
            _ => None,
        }
    }
}

/// Lib loader configuration
#[derive(Debug, Clone)]
pub struct LibLoaderConfig {
    /// Target ECMAScript version
    pub target: ScriptTarget,
    /// Specific lib files to include (overrides target-based defaults)
    pub lib: Option<Vec<LibFile>>,
    /// Whether to include DOM libs (for browser environments)
    pub include_dom: bool,
    /// No default lib
    pub no_lib: bool,
}

impl Default for LibLoaderConfig {
    fn default() -> Self {
        Self {
            target: ScriptTarget::ES5,
            lib: None,
            include_dom: true,
            no_lib: false,
        }
    }
}

/// Lib loader that provides built-in type declarations
pub struct LibLoader {
    config: LibLoaderConfig,
}

impl LibLoader {
    /// Create a new lib loader with the given configuration
    pub fn new(config: LibLoaderConfig) -> Self {
        Self { config }
    }

    /// Create a lib loader for the given target with default settings
    pub fn for_target(target: ScriptTarget) -> Self {
        Self::new(LibLoaderConfig {
            target,
            ..Default::default()
        })
    }

    /// Load all applicable lib declarations
    pub fn load(&self) -> LibDeclarations {
        if self.config.no_lib {
            return LibDeclarations::new();
        }

        let mut lib = LibDeclarations::new();

        // If specific libs are requested, use those
        if let Some(ref libs) = self.config.lib {
            for lib_file in libs {
                self.load_lib_file(&mut lib, *lib_file);
            }
            return lib;
        }

        // Otherwise, load based on target
        self.load_for_target(&mut lib, self.config.target);

        // Include DOM if requested
        if self.config.include_dom {
            lib.merge(dom::get_dom_declarations());
        }

        lib
    }

    /// Load declarations for a specific target
    fn load_for_target(&self, lib: &mut LibDeclarations, target: ScriptTarget) {
        // Always include ES5 base
        lib.merge(es5::get_es5_declarations());

        // Add features based on target
        if target >= ScriptTarget::ES2015 {
            lib.merge(es2015::get_es2015_declarations());
        }

        if target >= ScriptTarget::ES2020 {
            lib.merge(es2020::get_es2020_declarations());
        }

        if target >= ScriptTarget::ES2021 {
            lib.merge(es2020::get_es2021_declarations());
        }

        if target >= ScriptTarget::ES2022 {
            lib.merge(es2020::get_es2022_declarations());
        }

        if target >= ScriptTarget::ES2023 {
            lib.merge(es2020::get_es2023_declarations());
        }
    }

    /// Load a specific lib file
    fn load_lib_file(&self, lib: &mut LibDeclarations, lib_file: LibFile) {
        match lib_file {
            LibFile::ES5 => lib.merge(es5::get_es5_declarations()),
            LibFile::ES2015 |
            LibFile::ES2015Core |
            LibFile::ES2015Collection |
            LibFile::ES2015Symbol |
            LibFile::ES2015Iterable |
            LibFile::ES2015Generator |
            LibFile::ES2015Promise |
            LibFile::ES2015Proxy |
            LibFile::ES2015Reflect |
            LibFile::ES2015SymbolWellKnown => {
                lib.merge(es2015::get_es2015_declarations());
            }
            LibFile::ES2016 |
            LibFile::ES2016ArrayInclude => {
                // ES2016 adds Array.prototype.includes - included in ES2015 for simplicity
            }
            LibFile::ES2017 |
            LibFile::ES2017Object |
            LibFile::ES2017SharedMemory |
            LibFile::ES2017String |
            LibFile::ES2017Intl |
            LibFile::ES2017TypedArrays => {
                // ES2017 features
            }
            LibFile::ES2018 |
            LibFile::ES2018AsyncGenerator |
            LibFile::ES2018AsyncIterable |
            LibFile::ES2018Intl |
            LibFile::ES2018Promise |
            LibFile::ES2018Regexp => {
                // ES2018 features
            }
            LibFile::ES2019 |
            LibFile::ES2019Array |
            LibFile::ES2019Object |
            LibFile::ES2019String |
            LibFile::ES2019Symbol => {
                // ES2019 features
            }
            LibFile::ES2020 |
            LibFile::ES2020BigInt |
            LibFile::ES2020Intl |
            LibFile::ES2020Promise |
            LibFile::ES2020SharedMemory |
            LibFile::ES2020String |
            LibFile::ES2020SymbolWellKnown => {
                lib.merge(es2020::get_es2020_declarations());
            }
            LibFile::ES2021 |
            LibFile::ES2021Intl |
            LibFile::ES2021Promise |
            LibFile::ES2021String |
            LibFile::ES2021WeakRef => {
                lib.merge(es2020::get_es2021_declarations());
            }
            LibFile::ES2022 |
            LibFile::ES2022Array |
            LibFile::ES2022Error |
            LibFile::ES2022Intl |
            LibFile::ES2022Object |
            LibFile::ES2022SharedMemory |
            LibFile::ES2022String |
            LibFile::ES2022Regexp => {
                lib.merge(es2020::get_es2022_declarations());
            }
            LibFile::ES2023 |
            LibFile::ES2023Array => {
                lib.merge(es2020::get_es2023_declarations());
            }
            LibFile::ESNext => {
                // Include all ES features
                lib.merge(es2015::get_es2015_declarations());
                lib.merge(es2020::get_es2020_declarations());
                lib.merge(es2020::get_es2021_declarations());
                lib.merge(es2020::get_es2022_declarations());
                lib.merge(es2020::get_es2023_declarations());
            }
            LibFile::DOM |
            LibFile::DOMIterable |
            LibFile::DOMAsyncIterable => {
                lib.merge(dom::get_dom_declarations());
            }
            LibFile::WebWorker |
            LibFile::WebWorkerImportScripts => {
                // Web worker types - subset of DOM
            }
            LibFile::ScriptHost => {
                // Script host types
            }
            LibFile::Decorators |
            LibFile::DecoratorsLegacy => {
                // Decorator types
            }
        }
    }

    /// Get the list of default lib files for a target
    pub fn get_default_libs_for_target(target: ScriptTarget) -> Vec<LibFile> {
        let mut libs = vec![LibFile::ES5];

        if target >= ScriptTarget::ES2015 {
            libs.push(LibFile::ES2015);
        }
        if target >= ScriptTarget::ES2016 {
            libs.push(LibFile::ES2016);
        }
        if target >= ScriptTarget::ES2017 {
            libs.push(LibFile::ES2017);
        }
        if target >= ScriptTarget::ES2018 {
            libs.push(LibFile::ES2018);
        }
        if target >= ScriptTarget::ES2019 {
            libs.push(LibFile::ES2019);
        }
        if target >= ScriptTarget::ES2020 {
            libs.push(LibFile::ES2020);
        }
        if target >= ScriptTarget::ES2021 {
            libs.push(LibFile::ES2021);
        }
        if target >= ScriptTarget::ES2022 {
            libs.push(LibFile::ES2022);
        }
        if target >= ScriptTarget::ES2023 {
            libs.push(LibFile::ES2023);
        }

        libs.push(LibFile::DOM);
        libs.push(LibFile::DOMIterable);

        libs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_target_parsing() {
        assert_eq!(ScriptTarget::from_str("es5"), Some(ScriptTarget::ES5));
        assert_eq!(ScriptTarget::from_str("ES2015"), Some(ScriptTarget::ES2015));
        assert_eq!(ScriptTarget::from_str("es6"), Some(ScriptTarget::ES2015));
        assert_eq!(ScriptTarget::from_str("ESNext"), Some(ScriptTarget::ESNext));
        assert_eq!(ScriptTarget::from_str("invalid"), None);
    }

    #[test]
    fn test_lib_file_parsing() {
        assert_eq!(LibFile::from_str("es5"), Some(LibFile::ES5));
        assert_eq!(LibFile::from_str("lib.es2015.d.ts"), Some(LibFile::ES2015));
        assert_eq!(LibFile::from_str("dom"), Some(LibFile::DOM));
        assert_eq!(LibFile::from_str("es2015.promise"), Some(LibFile::ES2015Promise));
    }

    #[test]
    fn test_loader_for_es5() {
        let loader = LibLoader::for_target(ScriptTarget::ES5);
        let lib = loader.load();

        // Should have ES5 built-ins
        assert!(lib.find_interface("Array").is_some());
        assert!(lib.find_interface("Object").is_some());
        assert!(lib.find_interface("String").is_some());
        assert!(lib.find_interface("Number").is_some());
        assert!(lib.find_interface("Boolean").is_some());
        assert!(lib.find_interface("Function").is_some());
        assert!(lib.find_interface("Math").is_some());
        assert!(lib.find_interface("Date").is_some());
        assert!(lib.find_interface("RegExp").is_some());
        assert!(lib.find_interface("Error").is_some());
        assert!(lib.find_interface("JSON").is_some());

        // Should have DOM
        assert!(lib.find_interface("Document").is_some());
        assert!(lib.find_interface("Element").is_some());
        assert!(lib.find_interface("Window").is_some());
    }

    #[test]
    fn test_loader_for_es2015() {
        let loader = LibLoader::for_target(ScriptTarget::ES2015);
        let lib = loader.load();

        // Should have ES2015 additions
        assert!(lib.find_interface("Promise").is_some());
        assert!(lib.find_interface("Symbol").is_some());
        assert!(lib.find_interface("Map").is_some());
        assert!(lib.find_interface("Set").is_some());
        assert!(lib.find_interface("WeakMap").is_some());
        assert!(lib.find_interface("WeakSet").is_some());
    }

    #[test]
    fn test_loader_for_es2020() {
        let loader = LibLoader::for_target(ScriptTarget::ES2020);
        let lib = loader.load();

        // Should have ES2020 additions
        assert!(lib.find_interface("BigInt").is_some());
    }

    #[test]
    fn test_loader_no_lib() {
        let loader = LibLoader::new(LibLoaderConfig {
            no_lib: true,
            ..Default::default()
        });
        let lib = loader.load();

        // Should have no built-ins
        assert!(lib.interfaces.is_empty());
    }

    #[test]
    fn test_loader_no_dom() {
        let loader = LibLoader::new(LibLoaderConfig {
            include_dom: false,
            ..Default::default()
        });
        let lib = loader.load();

        // Should have ES5 but not DOM
        assert!(lib.find_interface("Array").is_some());
        assert!(lib.find_interface("Document").is_none());
    }

    #[test]
    fn test_loader_specific_libs() {
        let loader = LibLoader::new(LibLoaderConfig {
            lib: Some(vec![LibFile::ES5, LibFile::ES2015Promise]),
            include_dom: false,
            ..Default::default()
        });
        let lib = loader.load();

        // Should have ES5 and Promise
        assert!(lib.find_interface("Array").is_some());
        assert!(lib.find_interface("Promise").is_some());
        // But not DOM
        assert!(lib.find_interface("Document").is_none());
    }

    #[test]
    fn test_get_default_libs() {
        let libs_es5 = LibLoader::get_default_libs_for_target(ScriptTarget::ES5);
        assert!(libs_es5.contains(&LibFile::ES5));
        assert!(libs_es5.contains(&LibFile::DOM));
        assert!(!libs_es5.contains(&LibFile::ES2015));

        let libs_es2020 = LibLoader::get_default_libs_for_target(ScriptTarget::ES2020);
        assert!(libs_es2020.contains(&LibFile::ES5));
        assert!(libs_es2020.contains(&LibFile::ES2015));
        assert!(libs_es2020.contains(&LibFile::ES2020));
        assert!(libs_es2020.contains(&LibFile::DOM));
    }
}
