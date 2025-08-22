# pluau

Pluau provides high level Python bindings to Luau using PyO3/Maturin.

Asides from Buffer support, all Luau features should be broadly supported by pluau.

## Examples

### Creating a Lua VM and running a script

```py
import pluau
lua = pluau.Lua()
lua.set_memory_limit(1 * 1024 * 1024) # Optional: Set memory limit of the created Lua VM to 1MB
func = lua.load_chunk("return 2 + 2", name="example") # You can optionally set env as well to give the chunk its own custom global environment table (_G)
result = func()
print(result)  # [4]
```

### Tables

```py
tab = lua.create_table()
tab.push(123)
tab.set("key1", 456)

# Prints 1 123 followed by key1 456
for k, v in tab: 
    print("key", k, v)
print(len(tab)) # 1 (Lua/Luau only considers array part for length operator)

# Set a metatable
my_metatable = lua.create_table()
tab.set_metatable(my_metatable)

# Set the readonly property on the table (Luau-specific security feature) Luau s
tab.readonly = True

# The below will error now since the table is readonly
tab.set("key2", 789) # errors with "runtime error: attempt to modify a readonly table"
tab.readonly = False # make it writable again
tab.set("key2", 789) # works now
```

### Setting execution time limits

Luau offers interrupts which is a callback function that is called periodically during execution of Luau code. This can be used to implement execution time limits.

```py
import pluau
import time
start_time = time.time()
def interrupt(_: pluau.Lua):
    if time.time() - start_time > 1.0: # 1 second limit
        return pluau.VmState.Yield
    return pluau.VmState.Continue

lua = pluau.Lua()
lua.set_interrupt(interrupt)
func = lua.load_chunk("while true do end", name="infinite_loop")

# When using interrupts, the function should be made into a thread and then resumed. Otherwise, the yield will lead to a runtime error.
thread = lua.create_thread(func)
result = thread.resume() # Resume the thread with no arguments
print(result, thread.status) # Prints [] ThreadState.Resumable after 1 second
```