## Dynamic Binary Translator

This is an implementation of the toy virtual machine saw during the "Virtualization Techniques" course at TUM (WS 22/23).

The implementation uses LLVM as native code compiler, thanks to its ORC API (Just-in-time compilation). A dynamic basic block is compiled into native code when that block is executed only once. When the compilation is done, the compiled code will be cached for later uses.

If there exists a compiled dynamic basic block (also called *translation block*), then the native code will be invoked.

