#!/usr/bin/env sh

make
# echo "smoothing + binarise + inducing..."
# # ./pcfg_tool smooth -t 1 < ./training/large/training.mrg | \
# cat ./training/large/training.mrg | \
#     ./pcfg_tool binarise | \
#     ./pcfg_tool induce grammar
#
# echo "viterbi outside..."
# ./pcfg_tool outside ./training/large/grammar.rules ./training/large/grammar.lexicon grammar

echo "parsing..."
head -3 ./training/large/testsentences | \
    ./pcfg_tool parse ./training/large/grammar.rules ./training/large/grammar.lexicon  | \
    pv --line-mode > out.mrg
