﻿dxf-rs
======

A rust [crate](https://crates.io/crates/dxf) for reading and writing DXF CAD files.

# Usage

Put this in your `Cargo.toml`:

``` toml
[dependencies]
dxf = "0.2.1"
```

Or if you want [serde](https://github.com/serde-rs/serde) support, enable the `serialize` feature:

``` toml
[dependencies]
dxf = { version = "0.2.1", features = ["serialize"] }
```

> Note that `serde` support is intended to aid in debugging and since the serialized format is heavily
dependent on the layout of the structures, it may change at any time.

And finally add:

``` rust
extern crate dxf;
```

# Documentation

See the documentation [here](https://docs.rs/dxf/) on docs.rs.

# DXF Reference

Since I don't want to fall afoul of Autodesk's lawyers, this repo can't include the actual DXF documentation.  It can,
however contain links to the official documents that I've been able to scrape together.  For most scenarios the 2014
documentation should suffice, but all other versions are included here for backwards compatibility and reference
between versions.

[R10 (non-Autodesk source)](http://www.martinreddy.net/gfx/3d/DXF10.spec)

[R11 (differences between R10 and R11)](http://autodesk.blogs.com/between_the_lines/ACAD_R11.html)

[R12 (non-Autodesk source)](http://www.martinreddy.net/gfx/3d/DXF12.spec)

[R13 (self-extracting 16-bit executable)](http://www.autodesk.com/techpubs/autocad/dxf/dxf13_hlp.exe)

[R14](http://www.autodesk.com/techpubs/autocad/acadr14/dxf/index.htm)

[2000](http://www.autodesk.com/techpubs/autocad/acad2000/dxf/index.htm)

[2002](http://www.autodesk.com/techpubs/autocad/dxf/dxf2002.pdf)

[2004](http://download.autodesk.com/prodsupp/downloads/dxf.pdf)

[2005](http://download.autodesk.com/prodsupp/downloads/acad_dxf.pdf)

[2006](http://images.autodesk.com/adsk/files/dxf_format.pdf)

2007 (Autodesk's link erroneously points to the R2008 documentation)

[2008](http://images.autodesk.com/adsk/files/acad_dxf0.pdf)

[2009](http://images.autodesk.com/adsk/files/acad_dxf.pdf)

[2010](http://images.autodesk.com/adsk/files/acad_dxf1.pdf)

[2011](http://images.autodesk.com/adsk/files/acad_dxf2.pdf)

[2012](http://images.autodesk.com/adsk/files/autocad_2012_pdf_dxf-reference_enu.pdf)

[2013](http://images.autodesk.com/adsk/files/autocad_2013_pdf_dxf_reference_enu.pdf)

[2014](http://images.autodesk.com/adsk/files/autocad_2014_pdf_dxf_reference_enu.pdf)

These links were compiled from the archive.org May 9, 2013 snapshot of http://usa.autodesk.com/adsk/servlet/item?siteID=123112&id=12272454&linkID=10809853
(https://web.archive.org/web/20130509144333/http://usa.autodesk.com/adsk/servlet/item?siteID=123112&id=12272454&linkID=10809853)
