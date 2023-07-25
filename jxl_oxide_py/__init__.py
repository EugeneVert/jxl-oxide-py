from PIL import Image, ImageFile
from io import BytesIO

from .jxl_oxide_py import lib, ffi


def _accept(data):
    return (
        data[:2] == b"\xff\x0a"
        or data[:12] == b"\x00\x00\x00\x0c\x4a\x58\x4c\x20\x0d\x0a\x87\x0a"
    )


class JxlImageFile(ImageFile.ImageFile):
    format = "Jxl"
    format_description = "Jpeg XL image"
    __loaded = False

    def _open(self):
        self.fc = self.fp.read()

        self._decoder = lib.new(self.fc, len(self.fc))

        self._size = (
            lib.width(self._decoder),
            lib.height(self._decoder),
        )

        self.rawmode = ffi.string(lib.colorspace(self._decoder), 8).decode()
        self.mode = self.rawmode

        self.tile = []

    def load(self):
        if not self.__loaded:
            self._image = lib.image(self._decoder)
            self.data = ffi.buffer(self._image.data, self._image.len)

            self.__loaded = True

            if self.fp and self._exclusive_fp:
                self.fp.close()

            self.fp = BytesIO(self.data)
            self.tile = [("raw", (0, 0) + self.size, 0, self.rawmode)]

        return super().load()

    def close(self):
        lib.free_jxl_oxide(self._decoder)
        if self.__loaded:
            lib.free_array(self._image)
        super().close()


Image.register_open(JxlImageFile.format, JxlImageFile, _accept)
Image.register_extension(JxlImageFile.format, ".jxl")
Image.register_mime(JxlImageFile.format, "image/jxl")
