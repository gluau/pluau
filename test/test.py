import pluau
from pluau.utils import Argument

lua = pluau.Lua()
lua.sandbox(True)
print(lua.used_memory() / 1000) # Memory use in KB
lua.set_memory_limit(1 * 1024 * 1024) # Set memory limit to 1MB

def test_return(_: pluau.Lua, args: tuple[Argument]):
    x = args[0] if args else 0
    if not isinstance(x, int):
        raise TypeError("Expected an integer")
    if x < 0:
        raise ValueError("Negative value not allowed")
    return x+1

def test_return2(_: pluau.Lua, args: tuple[Argument]):
    x = args[0] if args else 0
    if not isinstance(x, int):
        raise TypeError("Expected an integer")
    if x < 0:
        raise ValueError("Negative value not allowed")
    return (x+1,)

print("Testing function with single return value")
fn = lua.create_function(test_return)
rets = fn.call([10, None])
assert(rets[0] == 11)
print("Testing function with tuple return value")
fn2 = lua.create_function(test_return2)
rets2 = fn2.call([10, None])
assert(rets2[0] == 11)

print(lua.used_memory() / 1000) # Memory use in KB

print("Testing function with no return value")
def test_return_none(_: pluau.Lua, args: tuple[Argument]):
    x = args[0] if args else 0
    if not isinstance(x, int):
        raise TypeError("Expected an integer")
    if x < 0:
        raise ValueError("Negative value not allowed")
    return None
fn3 = lua.create_function(test_return_none)
rets3 = fn3.call([10, None])
assert(rets3[0] is None)
assert(fn3 == fn3.deep_clone())
print(hex(fn3.pointer))

assert lua.get_registry_value("test_key") is None
lua.set_registry_value("test_key", 123)
assert lua.get_registry_value("test_key") == 123
lua.set_registry_value("test_key", None)
assert lua.get_registry_value("test_key") is None
print("Registry test passed")

def test_interrupt(_: pluau.Lua):
    raise pluau.RawError("Test interrupt") 

lua.set_interrupt(test_interrupt)
f = lua.load_chunk("while true do end", name="test_interrupt")
try:
    f.call([])
except RuntimeError as e:
    print(str(e))
    assert "Test interrupt" in str(e)

def test_interrupt2(_: pluau.Lua):
    return pluau.VmState.Yield

lua.set_interrupt(test_interrupt2)
try:
    f.call([])
except RuntimeError as e:
    # We aren't running in a created thread (its on Lua main thread)
    #
    # So we should get an error about yielding across C-call boundary
    assert "attempt to yield across metamethod/C-call boundary" in str(e)

print("Interrupt test passed")
