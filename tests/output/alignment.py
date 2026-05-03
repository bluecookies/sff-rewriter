def my_fn( *args, **kwargs ):
    pass

def my_function( *args, **kwargs ):
    pass

multi_line_array = [ 1, 2, 3, ]
multi_line_dict = { 'key1': 'value1', 'key2': 'value2', 'key3': 'value3', }
my_function( 1,
             '2',
             'value',
             1234567,
             "very long string that can't be broken",
             key='another value' )
my_function( 1,
             '2',
             'value',
             1234567,
             "very long string that can't be broken",
             key='another value', )
my_function( 1, '2', 'value', 1234567, ( 1, 2 ) )
my_function( 1,
             '2',
             'value',
             1234567,
             11111111111111111111111111111111111111111,
             ( 1, 2 ) )
my_function( 1,
             '2',
             'value',
             1234567,
             my_function( 'nested function call', 'on one line' ) )
my_function( 1,
             '2',
             'value',
             1234567,
             my_fn( 'nested function call, but this one',
                    'should break on two lines correctly' ) )


short_nested_dict = { 'key': { 'nested_key': 'nested_value' } }
long_nested_dict = { 'key': { 'nested_key': 'nested_value' },
                     'another_key': 'another_value',
                     'third_key': { 'third_key': 'third_value' }, }
nested_long_dict = { 'key': { 'foo1': 'bar1',
                              'foo2': 'bar2',
                              'foo3': 'bar3',
                              'foo4': 'bar4',
                              'foo5': 'bar5',
                              'foo6': 'bar6',
                              'foo7': 'bar7', } }
nested_long_array = { 'key': [ 'foo1',
                               'bar1',
                               'foo2',
                               'bar2',
                               'foo3',
                               'bar3',
                               'foo4',
                               'bar4',
                               'foo5',
                               'bar5',
                               'foo6',
                               'bar6',
                               'foo7', ] }
long_long_dict = { 'key': { 'nested_key': 'nested_value' },
                   'another_key': 'another_value',
                   'third_key': { 'third_key': 'third_value' },
                   'final_key': { 'foo1': 'bar1',
                                  'foo2': 'bar2',
                                  'foo3': 'bar3',
                                  'foo4': 'bar4',
                                  'foo5': 'bar5',
                                  'foo6': 'bar6',
                                  'foo7': 'bar7', } }
