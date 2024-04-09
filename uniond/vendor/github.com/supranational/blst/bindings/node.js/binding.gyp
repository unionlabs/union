# All pathnames (except for INTERMEDIATE_DIR) are relative to position
# of *this* file. If it's moved, the pathnames would have to be adjusted
# accordingly. It's also advised to move and customize blst_wrap.py, the
# script responsible for generating or copying pre-generated
# blst_wrap.cpp...
{
  'targets': [
    {
      'target_name': 'blst',
      'sources': [
        '<(INTERMEDIATE_DIR)/blst_wrap.cpp',
        '../../src/server.c',
      ],
      'include_dirs': [ '..' ],
      'cflags': [ '-fno-builtin-memcpy', '-fvisibility=hidden' ],
      'cflags_cc!': [ '-fno-exceptions' ],
      'xcode_settings': { 'GCC_ENABLE_CPP_EXCEPTIONS': 'YES' },
      'msvs_settings':  { 'VCCLCompilerTool': { 'ExceptionHandling': '1' } },
      'conditions': [
        [ 'OS=="win"', {
            'sources': [ '../../build/win64/*-x86_64.asm' ],
          }, {
            'sources': [ '../../build/assembly.S' ],
          }
        ],
        [ 'OS=="linux"', {
            'ldflags': [ '-Wl,-Bsymbolic' ],
          }
        ],
      ],
      'actions' : [
        {
          'action_name': 'blst_wrap',
          'variables': { 'cmd' : [ 'blst_wrap.py', '../blst.swg' ] },
          'inputs':  [ '<@(cmd)' ],
          'outputs': [ '<(INTERMEDIATE_DIR)/blst_wrap.cpp' ],
          'action':  [ 'python', '<@(cmd)', '<@(_outputs)' ],
        },
      ],
    },
  ]
}
