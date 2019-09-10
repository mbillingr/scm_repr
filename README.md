Findings:

- Reversing long cons lists is faster with garbage collection compared
  to simply leaking the memory. (On Linux; Windows needs further investigation)
  
- Simple object representation runs clearly slower.
- The difference between faster integers and cheaper pairs is not so big.