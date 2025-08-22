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
result = thread.resume() # This will return after 1 second with empty results
print(result, thread.status)