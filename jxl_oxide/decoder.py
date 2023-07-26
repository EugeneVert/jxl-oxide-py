from typing import Any
from ._internal import lib, ffi


class JxlOxideDecoder:
    def __init__(self, data) -> None:
        self._image = None
        self._decoder = lib.new(data, len(data))
        if self._decoder == ffi.NULL:
            raise_error()

    def size(self) -> tuple[int, int]:
        return (
            lib.width(self._decoder),
            lib.height(self._decoder),
        )

    def colorspace(self) -> str:
        return ffi.string(lib.colorspace(self._decoder), 8).decode()

    def image(self):
        self._image = self.wrapper(lib.image)
        return ffi.buffer(self._image.data, self._image.len)

    # -------------------------------------------------------------------------

    def wrapper(self, func) -> Any:
        """Wrapper for handling errors"""
        res = func(self._decoder)
        if res == ffi.NULL:
            raise_error()
        return res

    def __del__(self):
        lib.free_jxl_oxide(self._decoder)
        if self._image:
            lib.free_array(self._image)


def raise_error():
    """Raises ValueError with message recived from ffi"""
    buf_len = lib.last_error_length()
    buf = b"\0" * buf_len
    lib.last_error_message(buf, buf_len)
    err = "jxl-oxide: " + buf.decode()
    raise ValueError(err)
