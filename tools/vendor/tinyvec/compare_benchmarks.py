import os
import os.path
import json

comparisons = []

for (root, _dirs, files) in os.walk('target/criterion'):
    for file in files:
        if file == 'estimates.json' and root.endswith(
                'new') and 'TinyVec' in root:
            path = os.path.join(root, file)

            bench_name = path.split('/')[3]
            tinyvec_time = json.load(open(path))['mean']['point_estimate']

            path = path.replace('TinyVec', 'SmallVec')

            smallvec_time = json.load(open(path))['mean']['point_estimate']

            comparisons.append((bench_name, tinyvec_time / smallvec_time))

comparisons.sort(key=lambda x: x[1])
longest_name = max(len(c[0]) for c in comparisons)
for (name, ratio) in comparisons:
    # Undo the criterion name mangling
    name = name.replace('_[', '<[')
    name = name.replace(']___', ']>::')

    name = name.ljust(longest_name)
    print(f"{name} {ratio:.2f}")
