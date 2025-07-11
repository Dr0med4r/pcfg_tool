#!/usr/bin/env sh

make
# echo "smoothing + binarise + inducing..."
# # ./pcfg_tool smooth -t 1 < ./training/large/training.mrg | \
# cat ./training/large/training.mrg | \
#     ./pcfg_tool binarise | \
#     ./pcfg_tool induce grammar
#
echo "viterbi outside..."
./pcfg_tool outside ./training/small/grammar.rules ./training/small/grammar.lexicon grammar

echo "parsing..."
cat ./training/small/sentences | \
    ./pcfg_tool parse ./training/small/grammar.rules ./training/small/grammar.lexicon -a grammar.outside | \
    pv --line-mode > out.mrg
