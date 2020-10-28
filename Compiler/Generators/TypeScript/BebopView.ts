﻿
const hexDigits = "0123456789abcdef";
const asciiToHex = [
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  1,  2,  3,  4,  5,  6,  7,  8,  9,  0,  0,  0,  0,  0,  0,
    0, 10, 11, 12, 13, 14, 15,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0, 10, 11, 12, 13, 14, 15,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0
];

if (typeof require !== 'undefined') {
    if (typeof TextDecoder === 'undefined') (global as any).TextDecoder = require('util').TextDecoder;
    if (typeof TextEncoder === 'undefined') (global as any).TextEncoder = require('util').TextEncoder;
}

export class BebopView {
    private static textDecoder = new TextDecoder;
    private static textEncoder = new TextEncoder;
    private static writeBuffer: Uint8Array = new Uint8Array(256);
    private static writeBufferView: DataView = new DataView(BebopView.writeBuffer.buffer);
    private static instance: BebopView;
    public static getInstance(): BebopView {
        if (!BebopView.instance) {
            BebopView.instance = new BebopView();
        }
        return BebopView.instance;
    }

    private buffer: Uint8Array;
    private view: DataView;
    private byteToHex: string[] = []; // A lookup table: ['00', '01', ..., 'ff']
    index: number; // read pointer
    length: number; // write pointer

    private constructor() {
        this.buffer = BebopView.writeBuffer
        this.view = BebopView.writeBufferView
        this.index = 0;
        this.length = 0;
        for (const x of hexDigits) {
            for (const y of hexDigits) {
                this.byteToHex.push(x + y);
            }
        }
    }

    startReading(buffer: Uint8Array): void {
        this.buffer = buffer;
        this.view = new DataView(this.buffer.buffer, this.buffer.byteOffset, this.buffer.byteLength);
        this.index = 0;
        this.length = buffer.length;
    }

    startWriting(): void {
        this.buffer = BebopView.writeBuffer
        this.view = BebopView.writeBufferView
        this.index = 0;
        this.length = 0;
    }

    private growBy(amount: number): void {
        if (this.length + amount > this.buffer.length) {
            const data = new Uint8Array(this.length + amount << 1);
            data.set(this.buffer);
            this.buffer = data;
            this.view = new DataView(data.buffer);
        }
        this.length += amount;
    }

    skip(amount: number) {
        this.index += amount;
    }

    toArray(): Uint8Array {
        return this.buffer.subarray(0, this.length);
    }

    readByte():    number { return this.buffer[this.index++]; }
    readUint16():  number { const result = this.view.getUint16(this.index, true);    this.index += 2; return result; }
    readInt16():   number { const result = this.view.getInt16(this.index, true);     this.index += 2; return result; }
    readUint32():  number { const result = this.view.getUint32(this.index, true);    this.index += 4; return result; }
    readInt32():   number { const result = this.view.getInt32(this.index, true);     this.index += 4; return result; }
    readUint64():  bigint { const result = this.view.getBigUint64(this.index, true); this.index += 8; return result; }
    readInt64():   bigint { const result = this.view.getBigInt64(this.index, true);  this.index += 8; return result; }
    readFloat32(): number { const result = this.view.getFloat32(this.index, true);   this.index += 4; return result; }
    readFloat64(): number { const result = this.view.getFloat64(this.index, true);   this.index += 8; return result; }

    writeByte(value: number):    void { const index = this.length; this.growBy(1); this.buffer[index] = value; }
    writeUint16(value: number):  void { const index = this.length; this.growBy(2); this.view.setUint16(index, value, true); }
    writeInt16(value: number):   void { const index = this.length; this.growBy(2); this.view.setInt16(index, value, true); }
    writeUint32(value: number):  void { const index = this.length; this.growBy(4); this.view.setUint32(index, value, true); }
    writeInt32(value: number):   void { const index = this.length; this.growBy(4); this.view.setInt32(index, value, true); }
    writeUint64(value: bigint):  void { const index = this.length; this.growBy(8); this.view.setBigUint64(index, value, true); }
    writeInt64(value: bigint):   void { const index = this.length; this.growBy(8); this.view.setBigInt64(index, value, true); }
    writeFloat32(value: number): void { const index = this.length; this.growBy(4); this.view.setFloat32(index, value, true); }
    writeFloat64(value: number): void { const index = this.length; this.growBy(8); this.view.setFloat64(index, value, true); }

    readBytes(): Uint8Array {
        const length = this.readUint32(), start = this.index, end = start + length;
        this.index = end;
        return this.buffer.subarray(start, end);
    }

    writeBytes(value: Uint8Array): void {
        this.writeUint32(value.length);
        const index = this.length;
        this.growBy(value.length);
        this.buffer.set(value, index);
    }

    /**
     * Reads a null-terminated UTF-8-encoded string.
     */
    readString(): string {
        const start = this.index;
        while (this.buffer[this.index] !== 0) this.index++;
        return BebopView.textDecoder.decode(this.buffer.subarray(start, this.index++));

        /*
        let result = "";

        while (true) {
            let codePoint: number;

            // decode UTF-8
            const a = this.readByte();
            if (a < 0xC0) {
                codePoint = a;
            } else {
                const b = this.readByte();
                if (a < 0xE0) {
                    codePoint = ((a & 0x1F) << 6) | (b & 0x3F);
                } else {
                    const c = this.readByte();
                    if (a < 0xF0) {
                        codePoint = ((a & 0x0F) << 12) | ((b & 0x3F) << 6) | (c & 0x3F);
                    } else {
                        const d = this.readByte();
                        codePoint = ((a & 0x07) << 18) | ((b & 0x3F) << 12) | ((c & 0x3F) << 6) | (d & 0x3F);
                    }
                }
            }

            // strings are null-terminated
            if (codePoint === 0) {
                break;
            }

            // encode UTF-16
            if (codePoint < 0x10000) {
                result += String.fromCharCode(codePoint);
            } else {
                codePoint -= 0x10000;
                result += String.fromCharCode((codePoint >> 10) + 0xD800, (codePoint & ((1 << 10) - 1)) + 0xDC00);
            }
        }

        return result;
        */
    }

    /**
     * Writes a null-terminated UTF-8-encoded string.
     */
    writeString(value: string): void {
        // value.length * 3 is an upper limit for the space taken up by the string:
        // https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder/encodeInto#Buffer_Sizing
        // We add 1 for our trailing null byte.
        const maxBytes = value.length * 3 + 1;
        this.growBy(maxBytes);
        this.length -= maxBytes;
        const stats = BebopView.textEncoder.encodeInto(value, this.buffer.subarray(this.length));
        this.length += stats.written;
        this.buffer[this.length++] = 0;
        /*
        let codePoint: number;

        for (let i = 0; i < value.length; i++) {
            // decode UTF-16
            const a = value.charCodeAt(i);
            if (i + 1 === value.length || a < 0xD800 || a >= 0xDC00) {
                codePoint = a;
            } else {
                const b = value.charCodeAt(++i);
                codePoint = (a << 10) + b + (0x10000 - (0xD800 << 10) - 0xDC00);
            }

            // strings are null-terminated, so a string cannot contain a null character.
            if (codePoint === 0) {
                throw new Error("Cannot encode a string containing the null character");
            }

            // encode UTF-8
            if (codePoint < 0x80) {
                this.writeByte(codePoint);
            } else {
                if (codePoint < 0x800) {
                    this.writeByte(((codePoint >> 6) & 0x1F) | 0xC0);
                } else {
                    if (codePoint < 0x10000) {
                        this.writeByte(((codePoint >> 12) & 0x0F) | 0xE0);
                    } else {
                        this.writeByte(((codePoint >> 18) & 0x07) | 0xF0);
                        this.writeByte(((codePoint >> 12) & 0x3F) | 0x80);
                    }
                    this.writeByte(((codePoint >> 6) & 0x3F) | 0x80);
                }
                this.writeByte((codePoint & 0x3F) | 0x80);
            }
        }

        // write the null terminator
        this.writeByte(0);
        */
    }

    readGuid(): string {
        // Order: 3 2 1 0 - 5 4 - 7 6 - 8 9 - a b c d e f
        const b = this.byteToHex, a = this.buffer, i = this.index, d = '-';
        var s = b[a[i + 3]];
        s += b[a[i + 2]];
        s += b[a[i + 1]];
        s += b[a[i]];
        s += d;
        s += b[a[i + 5]];
        s += b[a[i + 4]];
        s += d;
        s += b[a[i + 7]];
        s += b[a[i + 6]];
        s += d;
        s += b[a[i + 8]];
        s += b[a[i + 9]];
        s += d;
        s += b[a[i + 10]];
        s += b[a[i + 11]];
        s += b[a[i + 12]];
        s += b[a[i + 13]];
        s += b[a[i + 14]];
        s += b[a[i + 15]];
        this.index += 16;
        return s;
    }

    writeGuid(value: string): void {
        const v = this.view, i = this.length;
        this.growBy(16);
        var p = 0, a = 0;
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        p += (value.charCodeAt(p) === 45) as any;
        v.setUint32(i, a, true);
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        p += (value.charCodeAt(p) === 45) as any;
        v.setUint16(i + 4, a, true);
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        p += (value.charCodeAt(p) === 45) as any;
        v.setUint16(i + 6, a, true);
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        p += (value.charCodeAt(p) === 45) as any;
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        v.setUint32(i + 8, a, false);
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        a = a<<4 | asciiToHex[value.charCodeAt(p++)];
        v.setUint32(i + 12, a, false);
    }

    writeEnum(value: any): void {
        var encoded = value as number;
        if (encoded === void 0) throw new Error("Couldn't convert enum value");
        this.writeUint32(encoded);
    }

    /**
     * Reserve some space to write a message's length prefix, and return its index.
     * The length is stored as a little-endian fixed-width unsigned 32-bit integer, so 4 bytes are reserved.
     */
    reserveMessageLength(): number {
        const i = this.length;
        this.growBy(4);
        return i;
    }

    /**
     * Fill in a message's length prefix.
     */
    fillMessageLength(position: number, messageLength: number): void {
        this.view.setUint32(position, messageLength, true);
    }

    /**
     * Read out a message's length prefix.
     */
    readMessageLength(): number {
        const result = this.view.getUint32(this.index, true);
        this.index += 4;
        return result;
    }
}
