from sys import (version_info,version)

def my_fn( *args, **kwargs ):
    pass

def my_function(*args, **kwargs):
    pass

# Check parentheses
my_function()

my_function( "already has a space" )
my_function(    "has multiple spaces"    )

my_lovely_tuple = (1, '2')
valid_singleton_tuple = (1,)


# Check brackets
foo = []
bar = [1]
baz = [1,2]
baz[1] = 3

# Check braces
hige = {}
piyo = {1,2,3}

# Check colon spacing
foo = { 'key':'value' }
baz[0:2] # should not be touched
def another_fn(_:int):  # space here
    if foo:     # no space here
        pass

# Check commas
foo = (1    ,)
foo = (   2, )
foo = ( 3,4,5 )
indexing = bar[1:,], bar[2:  ,]

some_var = 123
print(f"This brace should not be touched: {some_var = }")
print(f"But the bracket should: {bar[0] = }")
