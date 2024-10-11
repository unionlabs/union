#!/usr/bin/env python3

from cpp.generate import GenerateCpp
from csharp.generate import GenerateCSharp
from dart.generate import GenerateDart
from go.generate import GenerateGo
from java.generate import GenerateJava
from kotlin.generate import GenerateKotlin
from lobster.generate import GenerateLobster
from lua.generate import GenerateLua
from nim.generate import GenerateNim
from php.generate import GeneratePhp
from py.generate import GeneratePython
from rust.generate import GenerateRust
from swift.generate import GenerateSwift
from ts.generate import GenerateTs

# Run each language generation logic
GenerateCpp()
GenerateCSharp()
GenerateDart()
GenerateGo()
GenerateJava()
GenerateKotlin()
GenerateLobster()
GenerateLua()
GenerateNim()
GeneratePhp()
GeneratePython()
GenerateRust()
GenerateSwift()
GenerateTs()
