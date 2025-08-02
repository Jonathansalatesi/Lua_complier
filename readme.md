# Lua_complier

This project was programed by Rust, it is not only a complier but also a Virtual Machine, which is based on register not stack. 

- :stars:The instruction set of this VM is same with standard Lua VM. 
- The complier has 3 parts: lexer, parser, and code generator.
  - :bomb:Lexer is used for translating the Lua source code to tokens. 
  - :rocket:parser can turn token streams into AST(Abstract Syntax Tree).
  - :helicopter:code generator likes an abstract machine can note the information like jump address to generate digital code.
- This project has almost been finished(except for some minor bugs:bug:) in 1 yr:calendar:, and if possible:hourglass_flowing_sand:, all bugs would be fixed finally .

