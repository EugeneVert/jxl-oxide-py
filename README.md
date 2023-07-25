# jxl-oxide-py 

Experimental **Pillow** plugin to support decoding of JXL images via CFFI binding to the [jxl-oxide](https://github.com/tirr-c/jxl-oxide) crate.

Currently in early development. Bindings are made via ffi and may not be safe.


## Usage

```python
from PIL import Image
import jxl_oxide

img = Image.open("image.jxl")
```

## Build

```bash
maturin build -r
pip install {path to wheel}
```