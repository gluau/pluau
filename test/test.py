import pluau
from pluau.utils import Argument, Wrapper
import gc

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
rets = fn.call(10, None)
assert(rets[0] == 11)
rets = fn(11) # Using syntactic sugar
assert(rets[0] == 12)
print("Testing function with tuple return value")
fn2 = lua.create_function(test_return2)
rets2 = fn2.call(10, None)
assert(rets2[0] == 11)
rets2 = fn2(11) # Using syntactic sugar
assert(rets2[0] == 12)
assert(fn == fn.deep_clone()) # They are the same function
assert(fn2 == fn2.deep_clone()) # They are the same function
assert(fn != fn2) # They are different functions

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
rets3 = fn3.call(10, None)
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
    f.call()
except RuntimeError as e:
    print(str(e))
    assert "Test interrupt" in str(e)

def test_interrupt2(_: pluau.Lua):
    return pluau.VmState.Yield

lua.set_interrupt(test_interrupt2)
try:
    f.call()
except RuntimeError as e:
    # We aren't running in a created thread (its on Lua main thread)
    #
    # So we should get an error about yielding across C-call boundary
    assert "attempt to yield across metamethod/C-call boundary" in str(e)

print("Interrupt test passed")
cmem = lua.used_memory() / 1000 # Memory use in KB
print(cmem)
tab = lua.create_table()
assert tab.empty
assert tab.len() == 0 and len(tab) == 0
tab.set("key1", 123)
assert not tab.empty
assert tab.len() == 0 and len(tab) == 0 # hash part is the only thing we set above
tab.push(456)
assert tab.len() == 1 and len(tab) == 1
assert tab.pop() == 456
assert tab.raw_pop() is None
assert tab.get("key1") == 123
assert tab.get("key2") is None
tab.remove("key1")
assert tab.get("key1") is None
tab.set("key2", 789)
assert tab.get("key2") == 789
tab.set("key2", None)
assert tab.get("key2") is None
assert tab.empty

assert not tab.readonly
tab.readonly = True
assert tab.readonly
try:
    tab.set("key3", 456)
    raise RuntimeError("Should not be able to modify a readonly table")
except RuntimeError as e:
    assert "attempt to modify a readonly table" in str(e)
tab.readonly = False
assert not tab.readonly
tab.set("key3", 456)
assert tab.get("key3") == 456

tab.clear()
assert tab.empty
assert tab.len() == 0 and len(tab) == 0

tab.set_safeenv(True)
tab.set_safeenv(False)

tab = None
gc.collect()

assert lua.used_memory() / 1000 <= cmem + 10 # Memory use in KB

print("Thread test")

lua.remove_interrupt()

func3 = lua.load_chunk("local yielder = ...; yielder(); return 1, 2", name="test_func2")
print(func3)
thread = lua.create_thread(func3)
assert thread.status == pluau.ThreadState.Resumable
def yielder(lua: pluau.Lua, _args: tuple[Argument]):
    print("Yielding from yielder function")
    lua.yield_with("Yielded value")

yielderFunc = lua.create_function(yielder)
res = thread.resume(yielderFunc)
print(res, thread.status)
if res[0] != "Yielded value":
    raise RuntimeError("Expected 'Yielded value' but got: " + str(res[0]))
assert thread.status == pluau.ThreadState.Resumable
res = thread.resume()
print(res, thread.status)
assert res[0] == 1
assert res[1] == 2
assert thread.status == pluau.ThreadState.Finished

print("test for_each")
tab = lua.create_table()
tab.push(123)
tab.set("key1", 456)

for k, v in tab: 
    print("key", k, v)


wrapper = Wrapper(lua, secure_userdata=False)
class TestObject:
    def __init__(self):
        self.foo = 123
        self.blah = 393

code = lua.load_chunk("local obj = ...; print(obj, obj.foo, obj.blah, obj.bar); assert(obj.foo == 123); assert(obj.blah == 393)")
code(wrapper.wrap(TestObject()))

code = lua.load_chunk("local obj = ...; print(obj, obj.foo, obj.blah, obj.bar); assert(obj.foo == 123); assert(obj.blah == 393)")
code(wrapper.wrap({"foo": 123, "blah": 393}))