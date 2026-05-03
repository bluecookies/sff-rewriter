single_quoted = 'single quoted'
another_string = "double quoted -> should become single quoted"
string_in_map = {"key": "double quoted"}
dont_escape = "don't escape this"
f_strings = f"this can become {single_quoted}"
another_f_string = f"should stay {string_in_map['key']} because Python <3.12 doesn't support mixing quotes"
escaping = ["has escaped \"", "this too\'", 'and this \"']

"""here's a wild docstring"""
"""another one - these should not be touched"""
