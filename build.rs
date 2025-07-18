use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Link to system libraries
    println!("cargo:rustc-link-lib=z");  // Only link to zlib
    println!("cargo:rustc-link-lib=pthread");  // For pthread support
    
    // Tell cargo to invalidate the built crate whenever files change
    println!("cargo:rerun-if-changed=faigz/faigz_minimal.h");
    println!("cargo:rerun-if-changed=faigz/faigz_minimal.c");

    // Build the minimal faigz implementation
    cc::Build::new()
        .file("faigz/faigz_minimal.c")
        .include("faigz")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-unused-variable")
        .compile("faigz_minimal");

    // Build the wrapper C code that includes the faigz implementation
    cc::Build::new()
        .file("src/wrapper.c")
        .include("faigz")
        .compile("faigz_wrapper");

    // Generate bindings only if we can find the header
    if std::path::Path::new("faigz/faigz_minimal.h").exists() {
        let bindings = bindgen::Builder::default()
            .header("faigz/faigz_minimal.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .clang_arg("-Ifaigz")
            .generate();

        match bindings {
            Ok(bindings) => {
                bindings
                    .write_to_file(out_dir.join("bindings.rs"))
                    .expect("Couldn't write bindings!");
            }
            Err(e) => {
                eprintln!("Warning: Could not generate bindings: {e:?}");
                eprintln!("Creating minimal bindings...");

                // Create minimal bindings for compilation
                let minimal_bindings = r#"
                    #[repr(C)]
                    pub struct faidx_meta_t {
                        _unused: [u8; 0],
                    }
                    
                    #[repr(C)]
                    pub struct faidx_reader_t {
                        _unused: [u8; 0],
                    }
                    
                    #[allow(non_camel_case_types)]
                    pub type fai_format_options = ::std::os::raw::c_int;
                    #[allow(non_upper_case_globals)]
                    pub const FAI_NONE: fai_format_options = 0;
                    #[allow(non_upper_case_globals)]
                    pub const FAI_FASTA: fai_format_options = 1;
                    #[allow(non_upper_case_globals)]
                    pub const FAI_FASTQ: fai_format_options = 2;
                    
                    extern "C" {
                        pub fn faidx_meta_load(
                            filename: *const ::std::os::raw::c_char,
                            format: fai_format_options,
                            flags: ::std::os::raw::c_int,
                        ) -> *mut faidx_meta_t;
                        
                        pub fn faidx_meta_ref(meta: *mut faidx_meta_t) -> *mut faidx_meta_t;
                        pub fn faidx_meta_destroy(meta: *mut faidx_meta_t);
                        pub fn faidx_meta_nseq(meta: *const faidx_meta_t) -> ::std::os::raw::c_int;
                        pub fn faidx_meta_iseq(meta: *const faidx_meta_t, i: ::std::os::raw::c_int) -> *const ::std::os::raw::c_char;
                        pub fn faidx_meta_seq_len(meta: *const faidx_meta_t, seq: *const ::std::os::raw::c_char) -> i64;
                        pub fn faidx_meta_has_seq(meta: *const faidx_meta_t, seq: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
                        
                        pub fn faidx_reader_create(meta: *mut faidx_meta_t) -> *mut faidx_reader_t;
                        pub fn faidx_reader_destroy(reader: *mut faidx_reader_t);
                        pub fn faidx_reader_fetch_seq(
                            reader: *mut faidx_reader_t,
                            c_name: *const ::std::os::raw::c_char,
                            p_beg_i: i64,
                            p_end_i: i64,
                            len: *mut i64,
                        ) -> *mut ::std::os::raw::c_char;
                        pub fn faidx_reader_fetch_qual(
                            reader: *mut faidx_reader_t,
                            c_name: *const ::std::os::raw::c_char,
                            p_beg_i: i64,
                            p_end_i: i64,
                            len: *mut i64,
                        ) -> *mut ::std::os::raw::c_char;
                    }
                "#;

                std::fs::write(out_dir.join("bindings.rs"), minimal_bindings)
                    .expect("Couldn't write minimal bindings!");
            }
        }
    } else {
        eprintln!("Warning: faigz/faigz_minimal.h not found, using minimal bindings");
        // Create minimal bindings for compilation
        let minimal_bindings = r#"
            #[repr(C)]
            pub struct faidx_meta_t {
                _unused: [u8; 0],
            }
            
            #[repr(C)]
            pub struct faidx_reader_t {
                _unused: [u8; 0],
            }
            
            #[allow(non_camel_case_types)]
            pub type fai_format_options = ::std::os::raw::c_int;
            #[allow(non_upper_case_globals)]
            pub const FAI_NONE: fai_format_options = 0;
            #[allow(non_upper_case_globals)]
            pub const FAI_FASTA: fai_format_options = 1;
            #[allow(non_upper_case_globals)]
            pub const FAI_FASTQ: fai_format_options = 2;
            
            extern "C" {
                pub fn faidx_meta_load(
                    filename: *const ::std::os::raw::c_char,
                    format: fai_format_options,
                    flags: ::std::os::raw::c_int,
                ) -> *mut faidx_meta_t;
                
                pub fn faidx_meta_ref(meta: *mut faidx_meta_t) -> *mut faidx_meta_t;
                pub fn faidx_meta_destroy(meta: *mut faidx_meta_t);
                pub fn faidx_meta_nseq(meta: *const faidx_meta_t) -> ::std::os::raw::c_int;
                pub fn faidx_meta_iseq(meta: *const faidx_meta_t, i: ::std::os::raw::c_int) -> *const ::std::os::raw::c_char;
                pub fn faidx_meta_seq_len(meta: *const faidx_meta_t, seq: *const ::std::os::raw::c_char) -> i64;
                pub fn faidx_meta_has_seq(meta: *const faidx_meta_t, seq: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
                
                pub fn faidx_reader_create(meta: *mut faidx_meta_t) -> *mut faidx_reader_t;
                pub fn faidx_reader_destroy(reader: *mut faidx_reader_t);
                pub fn faidx_reader_fetch_seq(
                    reader: *mut faidx_reader_t,
                    c_name: *const ::std::os::raw::c_char,
                    p_beg_i: i64,
                    p_end_i: i64,
                    len: *mut i64,
                ) -> *mut ::std::os::raw::c_char;
                pub fn faidx_reader_fetch_qual(
                    reader: *mut faidx_reader_t,
                    c_name: *const ::std::os::raw::c_char,
                    p_beg_i: i64,
                    p_end_i: i64,
                    len: *mut i64,
                ) -> *mut ::std::os::raw::c_char;
            }
        "#;

        std::fs::write(out_dir.join("bindings.rs"), minimal_bindings)
            .expect("Couldn't write minimal bindings!");
    }
}
