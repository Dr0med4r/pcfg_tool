with copy for get_consequence and index:
 Performance counter stats for './pcfg_tool parse training/large/grammar.rules training/large/grammar.lexicon':

         31.070,70 msec task-clock:u                     #    0,998 CPUs utilized
                 0      context-switches:u               #    0,000 /sec
                 0      cpu-migrations:u                 #    0,000 /sec
           574.592      page-faults:u                    #   18,493 K/sec
    54.761.750.557      instructions:u                   #    0,94  insn per cycle
    58.346.171.287      cycles:u                         #    1,878 GHz
     9.557.848.740      branches:u                       #  307,616 M/sec
       165.606.808      branch-misses:u                  #    1,73% of all branches

      31,122285017 seconds time elapsed

      28,856361000 seconds user
       1,959424000 seconds sys

with references for get_consequence and index:
Performance counter stats for './pcfg_tool parse training/large/grammar.rules training/large/grammar.lexicon':

        31.235,69 msec task-clock:u                     #    0,999 CPUs utilized
                0      context-switches:u               #    0,000 /sec
                0      cpu-migrations:u                 #    0,000 /sec
          575.803      page-faults:u                    #   18,434 K/sec
   54.760.899.512      instructions:u                   #    0,93  insn per cycle
   58.671.738.895      cycles:u                         #    1,878 GHz
    9.557.720.672      branches:u                       #  305,987 M/sec
      165.515.025      branch-misses:u                  #    1,73% of all branches

     31,256069217 seconds time elapsed

     29,205274000 seconds user
      1,775413000 seconds sys

with references for get_consequence, index and add consequences
 Performance counter stats for './pcfg_tool parse training/large/grammar.rules training/large/grammar.lexicon':

         31.510,86 msec task-clock:u                     #    0,999 CPUs utilized
                 0      context-switches:u               #    0,000 /sec
                 0      cpu-migrations:u                 #    0,000 /sec
           575.548      page-faults:u                    #   18,265 K/sec
    54.757.360.565      instructions:u                   #    0,92  insn per cycle
    59.389.223.334      cycles:u                         #    1,885 GHz
     9.557.228.311      branches:u                       #  303,299 M/sec
       165.688.356      branch-misses:u                  #    1,73% of all branches

      31,547825388 seconds time elapsed

      29,308321000 seconds user
       1,940735000 seconds sys

with u32 everywhere
 Performance counter stats for './pcfg_tool parse training/large/grammar.rules training/large/grammar.lexicon':

         28.885,78 msec task-clock:u                     #    0,998 CPUs utilized
                 0      context-switches:u               #    0,000 /sec
                 0      cpu-migrations:u                 #    0,000 /sec
           500.454      page-faults:u                    #   17,325 K/sec
    54.784.836.530      instructions:u                   #    0,97  insn per cycle
    56.652.972.727      cycles:u                         #    1,961 GHz
     9.544.176.042      branches:u                       #  330,411 M/sec
       167.354.555      branch-misses:u                  #    1,75% of all branches

      28,930936766 seconds time elapsed

      27,059073000 seconds user
       1,578615000 seconds sys

with indexset instead of vec for data
 Performance counter stats for './pcfg_tool parse training/large/grammar.rules training/large/grammar.lexicon':

         45.791,41 msec task-clock:u                     #    0,996 CPUs utilized
                 0      context-switches:u               #    0,000 /sec
                 0      cpu-migrations:u                 #    0,000 /sec
           440.273      page-faults:u                    #    9,615 K/sec
    58.267.474.368      instructions:u                   #    0,93  insn per cycle
    62.988.289.912      cycles:u                         #    1,376 GHz
     9.876.209.178      branches:u                       #  215,678 M/sec
       170.763.192      branch-misses:u                  #    1,73% of all branches

      45,953421904 seconds time elapsed

      42,943991000 seconds user
       2,362123000 seconds sys

with binary rules 
 Performance counter stats for './pcfg_tool parse training/large/grammar.rules training/large/grammar.lexicon':

         25.479,97 msec task-clock:u                     #    0,999 CPUs utilized
                 0      context-switches:u               #    0,000 /sec
                 0      cpu-migrations:u                 #    0,000 /sec
           496.717      page-faults:u                    #   19,494 K/sec
    54.226.422.176      instructions:u                   #    1,03  insn per cycle
    52.829.835.680      cycles:u                         #    2,073 GHz
     9.276.978.874      branches:u                       #  364,089 M/sec
       164.410.194      branch-misses:u                  #    1,77% of all branches

      25,496739082 seconds time elapsed

      23,791011000 seconds user
       1,501884000 seconds sys
with btree and indexmap 
 Performance counter stats for './pcfg_tool parse training/large/grammar.rules training/large/grammar.lexicon':

         88.366,36 msec task-clock:u                     #    0,998 CPUs utilized
                 0      context-switches:u               #    0,000 /sec
                 0      cpu-migrations:u                 #    0,000 /sec
           234.861      page-faults:u                    #    2,658 K/sec
    44.631.765.942      instructions:u                   #    0,43  insn per cycle
   104.233.330.259      cycles:u                         #    1,180 GHz
     9.697.009.049      branches:u                       #  109,736 M/sec
       553.563.260      branch-misses:u                  #    5,71% of all branches

      88,527398207 seconds time elapsed

      85,700469000 seconds user
       1,254310000 seconds sys
with additional bitmap in weightmap for checking if set
 Performance counter stats for './pcfg_tool parse training/large/grammar.rules training/large/grammar.lexicon':

         21.782,99 msec task-clock:u                     #    0,999 CPUs utilized
                 0      context-switches:u               #    0,000 /sec
                 0      cpu-migrations:u                 #    0,000 /sec
           496.957      page-faults:u                    #   22,814 K/sec
    60.504.644.126      instructions:u                   #    1,24  insn per cycle
    48.790.908.663      cycles:u                         #    2,240 GHz
     9.468.363.529      branches:u                       #  434,668 M/sec
       161.196.727      branch-misses:u                  #    1,70% of all branches

      21,804966872 seconds time elapsed

      20,164341000 seconds user
       1,459286000 seconds sys
