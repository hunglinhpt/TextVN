import io

p = 'math/src/lexicon.rs'
s = io.open(p, encoding='utf-8').read()

BS = chr(92)
# Exact strings as they appear in the file (2 backslashes in \\mathrm lines)
ltx = BS + BS + 'mathrm{'

# --- si_row: remove {{ }} braces so macro expands to items ---
i = s.find('macro_rules! si_row')
j = s.find('macro_rules! si_tests_row')
assert 0 < i < j
block = s[i:j]
block = block.replace('$(,)? ]) => {{', '$(,)? ]) => {', 1)
block = block.replace(') ),*\n    }};', ') ),*\n    };', 1)
s = s[:i] + block + s[j:]

# --- si_tests_row: remove braces on both arms ---
i = s.find('macro_rules! si_tests_row')
j = s.find('macro_rules! si_table')
assert 0 < i < j
block = s[i:j]
block = block.replace('$units:tt) => {{', '$units:tt) => {', 1)
block = block.replace('si_tests_row!(@inner $pid, $psym, $units)\n        }\n    }};', 'si_tests_row!(@inner $pid, $psym, $units)\n        }\n    };', 1)
block = block.replace('$uvi:literal) ),* $(,)? ]) => {{', '$uvi:literal) ),* $(,)? ]) => {', 1)
block = block.replace('} )*\n    }};', '} )*\n    };', 1)
s = s[:i] + block + s[j:]

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('braces removed ok')
