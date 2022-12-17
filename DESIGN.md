# Design of the Cypher

Cypher Ideas
- random interspersement of message and boolean inverted message, so bit frequency is indeterminate 
    - this is vital to message security, as otherwise large ammounts of null bytes would obviously change behavior
    - (opt): add random noise to the bit count, to make it less obvious that it is being obscured
        - if this is not done, then the inverted message could be used as parody?
        - randomness would come from a system RNG (not shared info)
    - (opt): add random bytes to the data to obscure the length of messages (or use fixed-length messages?)
        - randomness would come from a system RNG (not shared info)
- (opt) do operations in base-3 to make it less intuitive and predictable?
- (opt, only for text, possibly not necesary) multiple representations for each letter so the expected frequency distribution becomes...
    - a uniform distrobution (preset/hardcoded into the encryption algorithm)
    - a random distrobution (uniform + noise). this would require the use of random data from the key (so that the multiple possibilities would be known to the decryptor)
- (opt) encode output in base-64 for ease of sending the message

- to deal with repetitive value issues, every time a message is created a new random word_pos is selected (system random, 0..2^68)
- the word_pos is encrypted with a word_pos of zero, and prepended to the output text.

good hash algorithm: https://github.com/BLAKE3-team/BLAKE3
good CSPRNG: https://rust-random.github.io/rand/rand_chacha/struct.ChaCha20Rng.html

Protocol Ideas
- Quantum key distrobution 
- messages shall be sent at one of many specific time internals, randomly selected to occur a random number of times each time the old selection finishes
- when there is no messages, they will be filled with text from a Markov chain based off of the "random article" Wikipedia button (and past user messages) and a true randomness source. these messages should be a comparable length to other normal messages
- these messages will have no defining factors, except for they must include some key sentance/phrase some number of times that is determinant on the message length.
- if to many messages build up in the queue, then the back pressure may force the sender to send faster. however, this change must not make it obvious that anything out of the ordinary is happening
