import builtins
from ._pluau import LightUserData, String, Function, Table, Thread, UserData, Vector, Buffer

type Argument = None | builtins.bool | LightUserData | builtins.int | builtins.float | Vector | builtins.str | String | Table | Function | Thread | UserData | Buffer

class FunctionWrapper():
    """Provides a nice wrapper around functions with argument validation."""
    ...