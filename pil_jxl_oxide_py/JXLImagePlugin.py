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
    __loaded = False

    def _open(self):
        self.fc = self.fp.read()

        self._decoder = jxl_oxide_py.lib.new(self.fc, len(self.fc))  # type: ignore

        self._size = (
            jxl_oxide_py.lib.width(self._decoder),  # type: ignore
            jxl_oxide_py.lib.height(self._decoder),  # type: ignore
        )

        self.rawmode = jxl_oxide_py.ffi.string(
            jxl_oxide_py.lib.colorspace(self._decoder), 8  # type: ignore
        ).decode()  # type: ignore
        self.mode = self.rawmode

        self.tile = []

    def load(self):
        if not self.__loaded:
            self._image = jxl_oxide_py.lib.image(self._decoder)  # type: ignore
            self.data = jxl_oxide_py.ffi.buffer(self._image.data, self._image.len)

            self.__loaded = True

            if self.fp and self._exclusive_fp:  # type: ignore
                self.fp.close()

            self.fp = BytesIO(self.data)  # type: ignore
            self.tile = [("raw", (0, 0) + self.size, 0, self.rawmode)]

        return super().load()

    def close(self):
        jxl_oxide_py.lib.free_jxl_oxide(self._decoder)  # type: ignore
        if self.__loaded:
            jxl_oxide_py.lib.free_array(self._image)  # type: ignore
        super().close()


Image.register_open(JxlImageFile.format, JxlImageFile, _accept)  # type: ignore
Image.register_extension(JxlImageFile.format, ".jxl")  # type: ignore
Image.register_mime(JxlImageFile.format, "image/jxl")  # type: ignore
