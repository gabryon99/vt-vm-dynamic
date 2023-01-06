/* Bytecode generator for Homework 2
 * (Lecture VT WS22/23, Weidendorfer)
 *
 * See init() function
 */
#include <memory.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// PRNG: get random 31-bit value if s is null. if s not null, set seed
int myrand(int s) {
  static int val = 1;
  if (s != 0) {
    val = s;
    return 0;
  }
  val = ((val * 1103515245U) + 12345U) & 0x7fffffff;
  return val;
}

// get random opcode between 1 and 5.
// ps[i] is calculated by init() below, and is the sum of relative
// probabilities up to opcode i+1, e.g. for 5-1-2-1-1 it is [5,6,8,9,10].
int get_random_opc(int ps[5]) {
  int i, v;
  v = myrand(0) % ps[4];
  for (i = 0; i < 4; i++)
    if (v < ps[i])
      break;
  return i + 1;
}

// Random generation of opcode sequence (with opcodes between 1 and 5) of
// given length <size> into buffer <buf>, putting HALT opcode 0 at end.
// * prob[] specifies relative probabilities,
//   e.g. 5-1-1-1-1 means opcode 1 with 5x higher probability than others
// * with same seed and probabilities, same sequence is generated
// * <rA> and <rL> point to int values which should be used at start values
//   for registers A and L when running the opcode sequence. With same
//   seed, they are guaranteed to be set to the same values.
void init(char *buf, int size, int prob[5], int seed, int *rA, int *rL) {
  int i, j, probsum, opc, ps[5];

  // prefix sums for getRandomOpcode()
  probsum = 0;
  for (i = 0; i < 5; i++) {
    probsum += prob[i];
    ps[i] = probsum;
  }

  myrand(seed);
  *rA = myrand(0) & 7;
  *rL = myrand(0) & 7;

  i = 0;
  while (i < size) {
    opc = get_random_opc(ps);
    if (opc == 5) { // BACK7
      if (i < 7)
        continue;                 // BACK7 not allowed in first 7 instr
      for (j = i - 6; j < i; j++) // no SETL in 6 instr before
        if (buf[j] == 4) {        // SETL
          while ((buf[j] == 4) || (buf[j] == 5))
            buf[j] = get_random_opc(ps);
        }
    }
    buf[i++] = opc;
  }
  buf[size - 1] = 0; // last is HALT
}
