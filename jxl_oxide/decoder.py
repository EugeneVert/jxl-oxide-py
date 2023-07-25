from ._internal import lib, ffi


class JxlOxideDecoder:
    def __init__(self, data) -> None:
        self._image = None
        self._decoder = lib.new(data, len(data))

    def size(self) -> tuple[int, int]:
        return (
            lib.width(self._decoder),
            lib.height(self._decoder),
        )

    def colorspace(self) -> str:
        return ffi.string(lib.colorspace(self._decoder), 8).decode()

    def image(self):
        self._image = lib.image(self._decoder)
        return ffi.buffer(self._image.data, self._image.len)

    def __del__(self):
        lib.free_jxl_oxide(self._decoder)
        if self._image:
            lib.free_array(self._image)
