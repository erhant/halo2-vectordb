const NUM_VECS = 50;
const DIM = 12;
const MAX = 20;

const arr = [];
for (let i = 0; i < NUM_VECS; i++) {
  const v = [];
  for (let j = 0; j < DIM; j++) {
    v.push(Math.random() * MAX);
  }
  arr.push(v);
}

// pretty print
console.log(JSON.stringify(arr, undefined, 2));
