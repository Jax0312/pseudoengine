# Pseudocode Syntax 
*In respect to Pseudocode Guide 2026*
## Comments
```
// This is a comment
```
Comments begin with '//' and continue till the end of the line

## Variable declaration
```
DECLARE <name> : <data type>
```
Available data types:
- INTEGER
- REAL
- BOOLEAN
- STRING
- DATE

CHAR type does not exist as CHAR and STRING are treated as the same during examination

Dates are in the format dd/mm/yyyy, literals are used like `3/14/2020`

Multiple variables of the same type can be declared with
```
DECLARE <var1>, <var2>, ... : <data type>
```

### Array declaration:
```
DECLARE <name> : ARRAY[<lower bound>:<upper bound>] OF <data type>
```
Note: The bounds are inclusive

Multi Dimensional arrays:
```
DECLARE <name> : ARRAY[<lb1>:<ub1>, <lb2>:<ub2>, ..., <lbn>:<ubn>] OF <data type>
```

### Accessing array elements
One-dimensional array:
```
myArray[index]
```
Multi-dimensional array:
```
myArray[index1, index2, ...]
```

## Variable assignment
```
<variable name> <- <value>
```
Assigning to an undefined variable will define and initialise it to the value assigned

### Assigning to array element
```
myArray[index] <- <value>
myArray[index1, index2, ...] <- <value>
```

## Constants
```
CONSTANT <name> = <value>
```


## Types
### Enum definition
```
TYPE <name> = (State1, State2, State3, ...)
```

### Pointer definition
```
TYPE <name> = ^<data type>
```

### Composite definition
```
TYPE <name>
    DECLARE <var1> : <data type 1>
    DECLARE <var2> : <data type 2>
    ...
ENDTYPE
```

### Declaration
```
DECLARE <variable name> : <type name>
```

### Assignment
```
// Enum
<enumVar> <- <state>

// Pointer
<pointerVar> <- <otherPointerVar>
<pointerVar> <- ^<otherVar>

// Composite
<compositeVar> <- <otherCompositeVar>
<compositeVar>.<member> <- <value>
```

### Accessing pointers
```
// Access value of ptrVar and store it in var
<var> <- <ptrVar>^

// Assign to variable referenced by pointer
<ptrVar>^ <- <value>
```

## Arithmetic operations
- \+ (Addition)
- \- (Subtraction)
- \* (Multiplication)
- / (Division)\
Result of division operator will always be of type `REAL`
- DIV - Integer division
- MOD - Modulus


## Comparison operators
- \> (Greater than)
- \>= (Greater than or equal to)
- < (Less than)
- <= (Less than or equal to)
- = (Equal to)
- <> (Not equal to)

## Logical operators
- AND
- OR
- NOT

## String concatenation
```
<str1> & <str2>
```

## Selection statements
If statement:
```
IF <condition> THEN
    ...
ELSE
    ...
ENDIF
```
- `ELSE` statements is optional


Case statement:
```
CASE OF <variable>
    <case 1> : ...
    <case 2> : ...
    <case 3> TO <case 4>: ... 
    ...
    <case n> : ...
    OTHERWISE: ...
ENDCASE
```
- `OTHERWISE` is optional
- cases may only be literals or a numerical range using `TO`

## Loops
While loop:
```
WHILE <condition>
    ...
ENDWHILE
```
Loops until condition is false

Repeat until loop:
```
REPEAT
    ...
UNTIL <condition>
```
- Loops until the condition is true
- Condition is checked at the end of an iteration

For loop:
```
FOR <counterVariable> <- <startValue> TO <stopValue> STEP <stepValue>
    ...
NEXT counterVariable
```
- Initialises counterVariable to startValue and loops till it reaches stopValue, incrementing it by stepValue each iteration if provided, otherwise incrementing it by 1
- `STEP <stepValue>` and `counterVariable` after `NEXT` are optional

## Procedures
Procedure with no paramaters:
```
PROCEDURE <name>
    ...
ENDPROCEDURE
```

Procedure with parameters:
```
PROCEDURE <name>([BYREF | BYVAL] <parameterName> : <data type>, <parameter2Name> : <data type>, ...)
    ...
ENDPROCEDURE
```
- `BYREF` - pass parameters by reference
- `BYVAL` - pass parameters by value
- If `BYREF` or `BYVAL` is not speified, `BYVAL` will be used as the default

### Calling procedures
No parameters:
```
CALL <procedureName>
OR
CALL <procedureName>()
```

With parameters:
```
CALL <procedureName>(<parameter1>, <parameter2>, ...)
```

## Functions
```
FUNCTION <name>(...) RETURNS <data type>
    ...
ENDFUNCTION
```
- Syntax for function parameters are identical to those of procedures
- Functions must have a `RETURN` statement that returns a value of the specified data type

### Calling functions
```
<functionName>(<parameter1>, <parameter2>, ...)
```
Function calls may be used inside expressions since they return a data type

### In-built functions
#### String functions
```
// Returns the length of a string
LENGTH(s : STRING) RETURNS INTEGER

// Returns the left n characters of a string
LEFT(s : STRING, n : INTEGER) RETURNS STRING

// Returns the right n characters of a string
RIGHT(s : STRING, n : INTEGER) RETURNS STRING

// Returns a string of length y starting at x
MID(s : STRING, x : INTEGER, y : INTEGER) RETURNS STRING

// Converts all alphabetical characters into uppercase
TO_UPPER(s : STRING) RETURNS STRING

// Converts all alphabetical characters into lowercase
TO_LOWER(s : STRING) RETURNS STRING

// Converts a number into a string
NUM_TO_STR(x : REAL) RETURNS STRING

// Converts a string into a REAL or INTEGER
STR_TO_NUM(s : STRING) RETURNS REAL
STR_TO_NUM(s : STRING) RETURNS INTEGER

// Returns whether a string is a valid number
IS_NUM(s : STRING) RETURNS BOOLEAN

// Returns the ASCII value of a character
ASC(c : STRING) RETURNS INTEGER

// Returns the character representation of an ASCII value
CHR(x : INTEGER) RETURNS STRING
```

#### Date functions
```
// Returns day of month
DAY(Date : DATE) RETURNS INTEGER

// Returns the month
MONTH(Date : DATE) RETURNS INTEGER

// Returns the year
YEAR(Date : DATE) RETURNS INTEGER

// Returns day of week(Starting on Sunday with value 1)
DAYINDEX(Date : DATE) RETURNS INTEGER

// Returns a date with corresponding day, month and year
SETDATE(Day, Month, Year : INTEGER) RETURNS DATE

// Returns current date
TODAY() RETURNS DATE
```

#### Misc functions
```
// Returns the integer part of a real(floor)
INT(x : REAL) RETURNS INTEGER
INT(x : INTEGER) RETURNS INTEGER

// Returns a random number from 0 to x inclusive
RAND(x : INTEGER) RETURNS REAL

// Checks if end of file is reached
EOF(filename : STRING) RETURNS BOOLEAN
```

## I/O
### Output to screen
```
OUTPUT <value>
```
Multiple values can be output at once with
```
OUTPUT <value1>, <value2>, ...
```

### Get user input
```
INPUT <variableName>
```
Gets user input and stores it in the given variable

### File Handling
```
// Open a file
// Modes are READ, WRITE, APPEND and RANDOM
// WRITE mode creates the file if it doesn't exist
OPENFILE <filename> FOR <mode>

// Reads one line form the file into the variable(requires READ mode)
READFILE <filename>, <variable>

// Writes a line with data provided(requires WRITE or APPEND mode)
WRITEFILE <filename>, <data>

// Serialise and writes a RECORD data type to file(requires RANDOM mode)
PUTRECORD <filename>, <data>

// read and de-serialise a RECORD data type from file(requires RANDOM mode)
GETRECORD <filename>, <variable>

// Move the file cursor to the specified line(requires RANDOM mode)
SEEK <filename>, <value>

// Closes the file
CLOSEFILE <filename>
```

### Class and Inheritance
Class with constructor:

Constructor is defined as a PROCEDURE with name NEW
```
CLASS Pet
    PRIVATE Name : STRING
        // 
        PUBLIC PROCEDURE NEW(GivenName : STRING)
        Name ← GivenName
    ENDPROCEDURE
ENDCLASS
```

Inheritance is denoted by the INHERITS keyword; superclass/parent class methods will be called using the
keyword SUPER, for example:
```
CLASS Cat INHERITS Pet
    PRIVATE Breed: INTEGER
    PUBLIC PROCEDURE NEW(GivenName : STRING, GivenBreed : STRING)
        SUPER.NEW(GivenName)
        Breed ← GivenBreed
    ENDPROCEDURE
ENDCLASS
```

To create an object, the following format is used:
```
<object name> ← NEW <class name>(<param1>, <param2> ...)
```
For example:
```
MyCat ← NEW Cat("Kitty", "Shorthaired")
```

`PUBLIC` methods and properties can be accessed from outside the class whereas `PRIVATE` methods and properties cannot


