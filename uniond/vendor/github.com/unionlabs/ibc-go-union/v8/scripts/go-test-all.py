#!/usr/bin/env python3
"""
The purpose of this script is to run unit tests for all go modules in the current 
directory. It works by recursively searching for all go.mod files in the directory and
subdirectories and then running `go test` on each of them.

It is not intended to be run directly, but rather to be called by the Makefile.
"""
import os
import subprocess

def require_env_var(name):
    """ Require an environment variable to be set. """
    value = os.environ.get(name, None)
    if value is None:
        print(f"Error: {name} environment variable is not set")
        exit(1)
    return value

def find_go_modules(directory):
    """ Find all go.mod files in the current directory and subdirectories. """
    go_mod_files = []
    for root, _, files in os.walk(directory):
        if 'go.mod' in files:
            go_mod_files.append(root)
    return go_mod_files

def run_tests_for_module(module, *runargs):
    """ Run the unit tests for the given module. """
    os.chdir(module)
    
    print(f"Running unit tests for {module}")

    # add runargs to test_command
    test_command = f'go test -mod=readonly {" ".join(runargs)} ./...'
    result = subprocess.run(test_command, shell=True)
    return result.returncode


def run_tests(directory, *runargs):
    """ Run the unit tests for all modules in dir. """
    print("Starting unit tests")

    # Find all go.mod files and get their directory names
    go_modules = find_go_modules(directory)

    exit_code = 0
    for gomod in sorted(go_modules):
        res = run_tests_for_module(gomod, *runargs)
        if res != 0:
            exit_code = res
    exit(exit_code)

if __name__ == '__main__':
    # Get the environment variables given to us by the Makefile
    ARGS = require_env_var('ARGS')
    EXTRA_ARGS = require_env_var('EXTRA_ARGS')
    TEST_PACKAGES = require_env_var('TEST_PACKAGES')

    run_tests(os.getcwd(), ARGS, EXTRA_ARGS, TEST_PACKAGES)
