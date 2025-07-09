#!/usr/bin/env sh

make
echo "smoothing + binarise + inducing..."
./pcfg_tool smooth -t 1 < ./training/large/training.mrg | \
    ./pcfg_tool binarise | \
    ./pcfg_tool induce grammar
echo "parsing..."
head -3 ./training/large/testsentences | \
    ./pcfg_tool parse grammar.rules grammar.lexicon -s | \
    pv --line-mode > /dev/null
