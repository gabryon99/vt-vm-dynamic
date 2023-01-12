## Dynamic Binary Translator

This is an implementation of the toy virtual machine saw during the "Virtualization Techniques" course at TUM (WS 22/23).

The implementation uses LLVM as native code compiler, thanks to its ORC API (Just-in-time compilation). A dynamic basic block is compiled into native code when that block is executed only once. When the compilation is done, the compiled code will be cached for later uses.

If there exists a compiled dynamic basic block (also called *translation block*), then the native code will be invoked.

### Personal Notes

It is really hard to find resources in how to implement a *dynamic binary translator* online. Therefore, I attach some useful resources:

* [Study of the techniques for emulation programming](http://www.codeslinger.co.uk/files/emu.pdf) by *a bored and boring guy* (*Victor Moya del Barrio*)
* [Binary Translation](https://lettieri.iet.unipi.it/virtualization/2018/binary-translation.pdf) by *prof. Giuseppe Lettieri* (University of Pisa)