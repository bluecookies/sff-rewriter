dtype = [ ( 'foo', 'U8' ),          # describe foo1
          ( 'foo', 'U8' ),
          ( 'foo', 'U8' ),          # describe foo2
          ( 'foo', 'U8' ),
          ( 'foo', 'U8' ),          # describe foo3
          ( 'foo', 'U8' ),          # describe foo4
          ( 'foo', 'U8' ),
          ( 'foo', 'U8' ),          # describe foo5
          ( 'foo', 'U8' ), ]        # describe foo6

dtype = [ ( 'foo'      , 'U8' ),          # describe foo7
          ( 'foobaz'   , 'U8' ),          # describe foobaz
          ( 'foobarbaz', 'U8' ),          # describe foobarbaz
          ( 'foofo'    , 'U8' ) ]         # describe foofo

weird = [ ( 'foo', 'bar' ),
          ( 'baz',
            # some comment
            'hige' ), ]

still_weird = [
    # comment
    ( 1, 2   ),
    ( 3, 114 ) ]         # comment

numeric_arrays = [ [ 1    , 22 , 31, 413 ],
                   [ 41   , 131, 13, 12  ],          # comment
                   [ 31231, 21 , 13, 1   ], ]
