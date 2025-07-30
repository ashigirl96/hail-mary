#!/bin/bash
# Fix Cobra completion bugs for zsh
# This script fixes:
# 1. Array append syntax in the generated completion file
# 2. Forces order preservation for command completions

COMPLETION_FILE="$1"

if [ -z "$COMPLETION_FILE" ]; then
    echo "Usage: $0 <completion-file>"
    exit 1
fi

if [ -f "$COMPLETION_FILE" ]; then
    # Fix 1: Fix the array append syntax
    sed -i.bak 's/completions+=${comp}/completions+=("${comp}")/' "$COMPLETION_FILE"
    
    # Fix 2: Force keep order by adding the directive
    # Replace "directive=0" with "directive=32" for subcommand completion
    # This ensures ShellCompDirectiveKeepOrder is always set
    sed -i.bak2 's/directive=0$/directive=32/' "$COMPLETION_FILE"
    
    # Alternative fix: Always use -V flag for _describe
    # This forces zsh to keep the order regardless of directive
    sed -i.bak3 's/eval _describe \$keepOrder/eval _describe -V/' "$COMPLETION_FILE"
    
    echo "Fixed completion file: $COMPLETION_FILE"
    echo "- Fixed array append syntax"
    echo "- Forced order preservation"
else
    echo "Completion file not found: $COMPLETION_FILE"
    exit 1
fi