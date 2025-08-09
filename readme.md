# Lua_complier

This project was programed by Rust, it is not only a compiler but also a Virtual Machine, which is based on register not stack. 

- :stars:The instruction set of this VM is same with standard Lua VM. 
- The compiler has 3 parts: lexer, parser, and code generator.
  - :bomb:Lexer is used for translating the Lua source code to tokens. 
  - :rocket:parser can turn token streams into AST(Abstract Syntax Tree).
  - :helicopter:code generator likes an abstract machine can note the information like jump address to generate digital code.
- This project has almost been finished(except for some minor bugs:bug:) in 1 yr:calendar:, and if possible:hourglass_flowing_sand:, all bugs would be fixed finally .
- I know the word "complier" is a spelling mistake, but I don't want to change it. It's just the name of the project. Consider it as if I've created a new word&#x1F601;.

## How to start?

- Firstly, we need to create a directory to store the source files, for example, you can make a directory `\Lua_complier\`, and download the source code in it.

- Secondly, run `cargo run` in cmd to compiler and build the project. This project only use a `regex `library except for `std` library.

  ```shell
  $\Lua_complier> cargo run
  warning: `Lua_compiler` (bin "Lua_compiler") generated 323 warnings (run `cargo fix --bin "Lua_compiler"` to apply 13 suggestions)
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.92s
       Running `target\debug\Lua_complier.exe`
  $\Lua_complier>
  ```

- Thirdly, go into the `\target\debug` directory  where the executable program resides, and now we can use this compiler happily.

  ```shell
  $\Lua_complier>cd .\target
  
  $\Lua_complier\target>
  $\Lua_complier\target>.\debug\Lua_complier.exe ..\example\hello_world.lua
  Hello, World!
  
  $\Lua_complier\target>.\debug\Lua_complier.exe ..\example\factorial.lua
  3628800
  
  $\Lua_complier\target>.\debug\Lua_complier.exe ..\example\fibonacci.lua
  987
  
  $\Lua_complier\target>.\debug\Lua_complier.exe ..\example\test.lua
  b       2
  c       3
  a       1
  1       a
  2       b
  3       c
  
  $\Lua_complier\target>.\debug\Lua_complier.exe -h
  Usage: lua <filename>
         lua [Optional] <filename>
  Options:
          -h or --help            helps
          -l or --asm             disassemble programs
          -v or --version         show version of compiler
  $\Lua_complier\target>.\debug\Lua_complier.exe --asm ..\example\hello_world.lua
  
  main <:0,0> (6 instructions)
  0+ params, 3 slots, 1 upvalues, 0 locals, 2 constants, 0 functions
          1       [-]     GETUPVAL        1 0
          2       [-]     LOADK           2 -1
          3       [-]     GETTABLE        0 1 2
          4       [-]     LOADK           1 -2
          5       [-]     CALL            0 2 1
          6       [-]     RETURN          0 1
  constants (2):
          1       "print"
          2       "Hello, World!"
  locals (0):
  upvalues (1):
          0               1       0
  
  $\Lua_complier\target>
  ```
  
We have prepared a lot examples for testing in directory `\example\`, and you can also use this to compile your Lua files.
  
**This compiler is still in developing, if you find the bugs, I warmly welcome any feedback questions you may have.**