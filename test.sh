#!/usr/bin/env sh

make
echo "smoothing + binarise + inducing..."
./pcfg_tool smooth -t 100 < ./training/large/training.mrg | \
    ./pcfg_tool binarise | \
    ./pcfg_tool induce grammar
echo "parsing..."
./pcfg_tool parse grammar.rules grammar.lexicon -s < ./training/large/testsentences | pv --line-mode > /dev/null
