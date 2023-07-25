from PIL import Image, ImageFile
from io import BytesIO
import jxl_oxide_py

_VALID_JXL_MODES = {"RGB", "RGBA", "L", "LA"}


def _accept(data):
    return (
        data[:2] == b"\xff\x0a"
        or data[:12] == b"\x00\x00\x00\x0c\x4a\x58\x4c\x20\x0d\x0a\x87\x0a"
    )


class JxlImageFile(ImageFile.ImageFile):
    format = "Jxl"
    format_description = "Jpeg XL image"

    def _open(self):
        self.fc = self.fp.read()

        self._decoder = jxl_oxide_py.lib.new(self.fc, len(self.fc))  # type: ignore

        self._size = (self._decoder.width, self._decoder.height)

        self.rawmode = jxl_oxide_py.ffi.string(
            jxl_oxide_py.lib.pil_colorspace(self._decoder), 8  # type: ignore
        ).decode()  # type: ignore
        self.mode = self.rawmode

        self.data = jxl_oxide_py.ffi.buffer(
            self._decoder.image, self._decoder.image_len
        )

        self.tile = [("raw", (0, 0) + self.size, 0, self.rawmode)]

    def load(self):
        if self.data is None:
            EOFError("no more frames")

        if self.fp:
            self.fp.close()

        self.fp = BytesIO(self.data)  # type: ignore

        return super().load()

    def __del__(self):
        print("DROP")
        jxl_oxide_py.lib.free(self._decoder)  # type: ignore


Image.register_open(JxlImageFile.format, JxlImageFile, _accept)  # type: ignore
Image.register_extension(JxlImageFile.format, ".jxl")  # type: ignore
Image.register_mime(JxlImageFile.format, "image/jxl")  # type: ignore
