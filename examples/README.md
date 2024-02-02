# Configs
In this directory are two example configurations: `maximal.cfg` and `minimal.cfg`. 
The former is a config with every setting turned on, and the latter is the least number of settings for a functional configuration. 
In practice, you probably want somewhere in the middle. In particular, `multitree variables` is a pretty specialized setting that you likely don't want.

# Wrappers
In this directory there are wrappers for two solvers: `cadical` for SAT and `maxcdcl` for MaxSAT.
In general, the wrapper should be an executable file. 
For bash this is natural, but for Python it is made executable with `#!/usr/bin/python` (or equivalent) in the first line.
In light of this, there is a `wrapper_template.py` script to get you started with adding your own solvers.

## Custom Solvers with Python
This is a description of how I would go about adding a new solver with Python. As long as the specs are met, it should work for any language.

### The Python script
The python script is fairly straightforward. It's responsibility is to call the solver binary and wait for it to finish, or, when recieving a SIGTERM forward it to the solver.
For input `sys.argv[1]` will be the w(cnf) file to pass to the solver and `sys.argv[2]` will be the intended location to write the logs.
An example can be seen in `wrapper_template.py`. The important parts are:
1) `parse_metric` where you parse a field from your solver such as time, blocked clauses, etc. This must return a single float.
2) `command` where you put the string corresponding to the command of the solver. For example `cadical` or `./my_solver`. 
It should not be given arguments as those are handled automatically.
3) `metric_name` where you put what splits will recognize your metric as. This must match one of the `tracked metrics` in your config.

You are able to add more metrics as desired! Just repeat creating functions, names, and adding them to the dictionary as shown on line 61.

