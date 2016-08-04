The WS2300 does not really have an API. All you can do is read memory cells as
BYTES, write NIBBLES (4bit) and set/unset bits. It is really very primitive.
So all you need is a good "map" of the needed memory areas and knowing the
simple communication.

ALL commands (except the "resync" or "reset" command which is just 0x06) start
by sending 4 address bytes. The address is the physical address of a 4-bit
nibble inside the station. You send the most significant first and end with the
least significant. Addresses are 4 bytes long. First address byte is either 0 or
1.

Address command bytes are coded like this. Matematically it is ``address_command
= 0x82 + hexdigit*4`` Or seen more visually.

    1 0 0 0 0 0 1 0
    +
    0 0 A3 A2 A1 A0 0 0
    =
    1 0 A3 A2 A1 A0 1 0

WS2300 returns 0S, 1S, 2S and 3S where S is a check digit calculated as
``(command-0x82)/4``. I.e. S is the hex digits of the address.

The last command bytes is as follows.

WHEN READING you ask for N bytes. Note that data is read a bytes - two nibbles
at a time. If you need 5 addresses you have to read 6 by asking for 3 bytes. So
the last command byte is the number of databytes requested. It is coded as
``0xC2 + N*4``. Max value is 15.

Or visually

    1 1 0 0 0 0 1 0
    +
    0 0 N3 N2 N1 N0 0 0
    =
    1 1 N3 N2 N1 N0 1 0

The station returns the acknowledge 3X where X is the number of data bytes to
follow (excl checksum byte). The station then sends all the requested databytes
and ends with an extra checksum byte which is calculated as the sum of the N
data bytes. Only the least significant 8 bits are used.

Writing nibbles is similar. Addressing is the same 4 address command. You then
follow with 1 and up to 15 data command bytes. They are coded as ``data_command=
0x42 + (data_nibble*4)``

    0 1 0 0 0 0 1 0
    +
    0 0 D3 D2 D1 D0 0 0
    =
    0 1 D3 D2 D1 D0 1 0

The station will acknowledge every written nibble write command by returning
``(0x10 + hexdigit)``. After the last data byte is written you send can either
send 06 init command (acknowledged by 0x02) or a new read/write address
(0x82/0x83) which is then acknowledged by 0x00/0x01. It is probably a good idea
to end the writing in some controlled way so that no more data is written to the
station.

Setting/resetting bits. Same method as for writing nibbles except data command
have the following format. Command for setting a bit (numbered from 0 to 3) is
``0x12+ (bit_no*4)`` and the acknowledge is ``0x04+bit_no``. Command for
resetting a bit is ``0x32+(bit_no*4)`` and the acknowledge is ``0x0C+bit_no``.

SETTING

    0 0 0 1 0 0 1 0
    +
    0 0 0 0 B1 B0 0 0
    =
    0 0 0 1 B1 B0 1 0

UNSETTING

    0 0 1 1 0 0 1 0
    +
    0 0 0 0 B1 B0 0 0
    =
    0 0 1 1 B1 B0 1 0

That was the basics. It has been posted before in this thread but I thought it
would be nice to have it all at one place. To get the history data you only
need the read command. First read 10 bytes (20 nibbles) from address 06B2 and
get current history settings.

HISTORY SETTINGS

    06B2 History saving interval: Binary nibble 0 [minutes] Coded as minutes - 1.
    06B3 History saving interval: Binary nibble 1 [minutes]
    06B4 History saving interval: Binary nibble 2 [minutes]
    06B5 Countdown to next saving: Binary nibble 0 [minutes] Minutes left - 1
    06B6 Countdown to next saving: Binary nibble 1 [minutes]
    06B7 Countdown to next saving: Binary nibble 2 [minutes]
    06B8 Time last record, minutes BCD 1s
    06B9 Time last record, minutes BCD 10s
    06BA Time last record, hours BCD 1s
    06BB Time last record, hours BCD 10s
    06BC Date last record, BCD day 1s
    06BD Date last record, BCD day 10s
    06BE Date last record, BCD month 1s
    06BF Date last record, BCD month 10s
    06C0 Date last record, BCD year 1s
    06C1 Date last record, BCD year 10s
    06C2 Pointer to last written Record: Binary nibble 0 [Range 00-AE]
    06C3 Pointer to last written Record: Binary nibble 1
    06C4 Number of Records: Binary nibble 0 [Range 00-AF]
    06C5 Number of Records: Binary nibble 1

The address area 06B4-06B2 is the interval. You can change this using the write
command if you want. Remember -1. The address area 06B7-06B5 is time till next
data record. The station counts from the time set in 06B4-06B2. When it reaches
zero another minute passes and then the data is taken and the value put back to
the start value. This value can also be set by your program.

The 06C1-06B8 area is the time stamp of the last record. Your software will
need to use this and the interval to calculate how many data points you need to
read to catch up with where you were last.

06C5-06C4 counts the number of valid data records. You can set this to zero when
you start - you do not have to. When it reaches AF is stays at this value
indicating that the entire ring buffer is full.

The 06C3-06C2 is a pointer to the LAST WRITTEN data record. It can have values
from 00 to AE. When it reaches AE it goes to 00.

Each record is 19 nibbles. So you must read 10 bytes at a time and throw the
last nibble away. If you want to limit data transfer you can read 15 at a time
and make the software smart. That is up to you.

The first record 0 is at address 0x06C6. Record 1 is ``0x0C6 + 19= 06D9``. Last
record is record 0xAE. So you calculate the last data record the formular is
``0xC6C + N*19`` where N is the pointer in address 06C3-06C2.  The rest is simple
programming.

Note. If you set both pointer and number of records to zero, the next record
become 1. The pointer points to the last record written. Not the next to be
written.

A summary of the data records nibble 0 to 18. (It is easier to do address
calculation when you start with 0 - it is not only because I am a nerd.

```
4,3,2,1,0: Indoor and outdoor temperature
Tindoor = (value % 1000)/10 - 30 [C]
Toutdoor = (value - (value % 1000))/10000 - 30 [C]
Where % is the modulus operator.
```

```
9,8,7,6,5: Air Pressure (absolute) and Indoor Humidity.
Pressure= 1000 + (value % 10000)/10. If pressure is greater than or equal to 1500 then you subtract 1000.
Indoor humidity =(value-(value % 10000))/10000
```

```
11,10: Outdoor Humidity in plain human readable BCD
```

```
14,13,12: Rain. (RAINCOUNTn)
The value is binary and steps 0.518 mm/step.
The absolute value does not seem to be related to anything else than an internal "household" value inside the station.
Every period the current 12-bit rain count value is stored as history data.
You use it by keeping a reference total rain value RAINref, the corresponding reference count RAINCOUNTref.
RAINtotal = RAINref + (RAINCOUNTn - RAINCOUNTref)*0.518 [mm]
```

```
17-16-15: Windspeed = value in binary / 10 [m/s]
```

```
18: Wind direction = value * 22.5 degrees. North is 0 and degrees are clockwise on the circle.

```
