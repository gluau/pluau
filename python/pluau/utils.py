import builtins
from ._pluau import LightUserData, String, Function, Table, Thread, UserData, Vector, Buffer, Lua, WeakLua, RawError
from .args import Argument

def conv_types(v: Argument, depth: int = 0, max_depth: int = 20):
    """Converts between primitive Luau builtin types and python built in types where possible"""
    if depth > max_depth:
        return
    if isinstance(v, Vector):
        return [v.x, v.y, v.z]
    elif isinstance(v, String):
        return str(v)
    elif isinstance(v, Table):
        dict_r = {}
        for key, value in v:
            key_conv = conv_types(key, depth + 1, max_depth)
            value_conv = conv_types(value, depth + 1, max_depth)
            if key_conv is not None and not getattr(key_conv, "__hash__", None):
                raise RawError(f"cannot convert key {key} of type {type(key).__name__} to a primitive type")
            dict_r[key_conv] = value_conv
        return dict_r
    else:
        # Passthrough unsupported types
        return v

class Wrapper:
    """
    Normal pluau can only map primitive Python types to Luau types.

    This class is a utility class to allow for mapping arbitrary Python objects, first to Luau
    primitives (if possible), and if not possible, to an opaque Luau userdata.

    If ``secure_userdata`` is ``False``, the created opaque userdata will have its methods+fields
    proxied directly. Otherwise, no fields/methods will be exposed on the userdata
    """
    __wlua: WeakLua # Weak reference to the Lua state
    max_stack_depth: int
    secure_userdata: bool # Whether the userdata should be fully opaque (secure) or have proxy fields/methods
    def __init__(self, lua: Lua, max_stack_depth: int = 20, secure_userdata: bool = True):
        """Constructs an new Object wrapper for the given Lua state."""
        self.__wlua = lua.weak()
        self.max_stack_depth = max_stack_depth
        self.secure_userdata = secure_userdata
    
    def __acquire_lua(self) -> Lua:
        lua = self.__wlua.upgrade()
        if lua is None:
            raise RuntimeError("The Lua state has been garbage collected/destroyed")
        return lua

    def wrap(self, obj: object) -> Argument:
        """Wraps the given Python object to a Luau type."""
        return self.__wrap(obj)
    
    def __wrap(self, obj: object, stack_depth: int = 0, lua: Lua | None = None) -> Argument:
        if stack_depth > self.max_stack_depth:
            raise RecursionError("Maximum stack depth exceeded while wrapping object")
        if obj is None:
            return None
        elif isinstance(obj, (bool, LightUserData, int, float, Vector, str, String, Table, Function, Thread, UserData, Buffer)):
            return obj
        elif isinstance(obj, (list, tuple)):
            lua = lua or self.__acquire_lua()
            tbl = lua.create_table_with_capacity(len(obj), 0)
            for v in obj:
                tbl.push(self.__wrap(v, stack_depth+1, lua=lua))
            return tbl
        elif isinstance(obj, dict):
            lua = lua or self.__acquire_lua()
            tbl = lua.create_table_with_capacity(0, len(obj))
            for k, v in obj.items():
                k_wrapped = self.__wrap(k, stack_depth+1, lua=lua)
                v_wrapped = self.__wrap(v, stack_depth+1, lua=lua)
                tbl.set(k_wrapped, v_wrapped)
            return tbl
        else:
            lua = lua or self.__acquire_lua()
            return Object(obj, secure_userdata=self.secure_userdata).create(lua)
        
class Object:
    """
    Given an arbitrary python object, creates a opaque userdata wrapper
    
    If ``secure_userdata`` is ``False``, the created opaque userdata will have its methods+fields
    proxied directly. Otherwise, no fields/methods will be exposed on the userdata
    """
    __obj: object
    __typename: str
    secure_userdata: bool
    def __init__(self, obj: object, secure_userdata: bool):
        self.__obj = obj
        self.__typename = type(obj).__name__
        self.secure_userdata = secure_userdata

    @staticmethod
    def __index_mt(lua: Lua, args: list[Argument]):
        """Index metamethod"""
        if len(args) != 2:
            raise RawError("internal error in Object.__index_mt, not enough args passed")
        ud = args[0]
        key = conv_types(args[1])
        if not isinstance(ud, UserData):
            raise RawError("internal error in Object.__index_mt, userdata not passed in as first argument")
        dyn_data = ud.data()
        attr = getattr(dyn_data, key, None) # type: ignore 
        return Wrapper(lua, secure_userdata=False).wrap(attr)
    
    @staticmethod
    def __newindex_mt(lua: Lua, args: list[Argument]):
        if len(args) != 3:
            raise RawError("internal error in Object.__newindex_mt, not enough args passed")
        ud = args[0]
        key = conv_types(args[1])
        value = conv_types(args[2])
        if not isinstance(ud, UserData):
            raise RawError("internal error in Object.__newindex_mt, userdata not passed in as first argument")
        dyn_data = ud.data()
        setattr(dyn_data, key, value) # type: ignore 

    def create(self, lua: Lua) -> UserData:
        """Creates the object for the given Luau reference"""
        # Just set __metatable to false and __index to {}
        ud_mt = lua.create_table_with_capacity(0, 2 if self.secure_userdata else 4)
        ud_mt.set("__metatable", False)
        ud_mt.set("__type", self.__typename)
        if not self.secure_userdata:
            ud_mt.set("__index", lua.create_function(Object.__index_mt))
            ud_mt.set("__newindex", lua.create_function(Object.__newindex_mt))

        return lua.create_userdata(self.__obj, ud_mt)
