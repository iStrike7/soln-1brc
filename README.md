# 1BRC - SAFE RUST AND NO EXTERNAL LIBRARY SOLUTION
This is a safe Rust solution to the 1brc that uses no external libraries

The approach to this solution was mostly inspired by Jon Gjenset's solution:  
https://www.youtube.com/watch?v=tCY7p6dVAGE&t=36209s  
https://github.com/jonhoo/brrr

## DIFFERENCES WITH JONHOO/BRRR
1. Uses BufReader read_until instead of mmap
2. Slightly better temperature parsing
3. No unsafe code
4. No SIMD or Memchr

## PERFORMANCE
Benchmarked with `time` on linux  
This solution on my machine takes 9.4 secs while jonhoo/brrr takes around 7.2 secs  
Interestingly, other popular solutions seem to result in approx the same ~7.2 secs  

## FURTHER SCOPE
1. Use a custom hasher (without presumptions about the dataset)
2. Scout other rust solutions (such as https://curiouscoding.nl/posts/1brc) for tricks that can be employed in safe rust
3. Implement with external libraries (safe rust) and compare
4. Implement and compare exactly how much of the performance difference is from mmap/simd and what is the remaining performance gap (isolating the benefit from extensive unsafe code)