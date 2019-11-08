title: The SparSDR OOT Module
brief: Utilities for configuring and using SparSDR compression on a USRP
tags: # Tags are arbitrary, but look at CGRAN what other authors are using
  - sdr
author:
  - Sam Crow <scrow@eng.ucsd.edu>
copyright_owner:
  - The Regents of the University of California
license:
repo: https://github.com/ucsdsysnet/sparsdr
#website: <module_website> # If you have a separate project website, put it here
#icon: <icon_url> # Put a URL to a square image here that will be used as an icon on CGRAN
---
This module provides utilities for cofiguring and using SparSDR compression
with a USRP. It includes:

* A utility class for changing the compression settings of a USRP
* A block that detects average samples sent by a USRP
* A hierarchical block that receives samples from a USRP and writes them to
a file
* A hierarchical block that can be configured to read signals from files,
optionally resample them, and send them to various sniffers and decoders
* A ready-to-use application that receives samples from a USRP and writes them
to a file, automatically restarting compression if the compression process
experiences overflow and stops
