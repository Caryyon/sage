var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __commonJS = (cb, mod) => function __require() {
  return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));

// node_modules/base64-js/index.js
var require_base64_js = __commonJS({
  "node_modules/base64-js/index.js"(exports) {
    "use strict";
    exports.byteLength = byteLength;
    exports.toByteArray = toByteArray;
    exports.fromByteArray = fromByteArray2;
    var lookup = [];
    var revLookup = [];
    var Arr = typeof Uint8Array !== "undefined" ? Uint8Array : Array;
    var code = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for (i = 0, len = code.length; i < len; ++i) {
      lookup[i] = code[i];
      revLookup[code.charCodeAt(i)] = i;
    }
    var i;
    var len;
    revLookup["-".charCodeAt(0)] = 62;
    revLookup["_".charCodeAt(0)] = 63;
    function getLens(b64) {
      var len2 = b64.length;
      if (len2 % 4 > 0) {
        throw new Error("Invalid string. Length must be a multiple of 4");
      }
      var validLen = b64.indexOf("=");
      if (validLen === -1) validLen = len2;
      var placeHoldersLen = validLen === len2 ? 0 : 4 - validLen % 4;
      return [validLen, placeHoldersLen];
    }
    function byteLength(b64) {
      var lens = getLens(b64);
      var validLen = lens[0];
      var placeHoldersLen = lens[1];
      return (validLen + placeHoldersLen) * 3 / 4 - placeHoldersLen;
    }
    function _byteLength(b64, validLen, placeHoldersLen) {
      return (validLen + placeHoldersLen) * 3 / 4 - placeHoldersLen;
    }
    function toByteArray(b64) {
      var tmp;
      var lens = getLens(b64);
      var validLen = lens[0];
      var placeHoldersLen = lens[1];
      var arr = new Arr(_byteLength(b64, validLen, placeHoldersLen));
      var curByte = 0;
      var len2 = placeHoldersLen > 0 ? validLen - 4 : validLen;
      var i2;
      for (i2 = 0; i2 < len2; i2 += 4) {
        tmp = revLookup[b64.charCodeAt(i2)] << 18 | revLookup[b64.charCodeAt(i2 + 1)] << 12 | revLookup[b64.charCodeAt(i2 + 2)] << 6 | revLookup[b64.charCodeAt(i2 + 3)];
        arr[curByte++] = tmp >> 16 & 255;
        arr[curByte++] = tmp >> 8 & 255;
        arr[curByte++] = tmp & 255;
      }
      if (placeHoldersLen === 2) {
        tmp = revLookup[b64.charCodeAt(i2)] << 2 | revLookup[b64.charCodeAt(i2 + 1)] >> 4;
        arr[curByte++] = tmp & 255;
      }
      if (placeHoldersLen === 1) {
        tmp = revLookup[b64.charCodeAt(i2)] << 10 | revLookup[b64.charCodeAt(i2 + 1)] << 4 | revLookup[b64.charCodeAt(i2 + 2)] >> 2;
        arr[curByte++] = tmp >> 8 & 255;
        arr[curByte++] = tmp & 255;
      }
      return arr;
    }
    function tripletToBase64(num) {
      return lookup[num >> 18 & 63] + lookup[num >> 12 & 63] + lookup[num >> 6 & 63] + lookup[num & 63];
    }
    function encodeChunk(uint8, start, end) {
      var tmp;
      var output = [];
      for (var i2 = start; i2 < end; i2 += 3) {
        tmp = (uint8[i2] << 16 & 16711680) + (uint8[i2 + 1] << 8 & 65280) + (uint8[i2 + 2] & 255);
        output.push(tripletToBase64(tmp));
      }
      return output.join("");
    }
    function fromByteArray2(uint8) {
      var tmp;
      var len2 = uint8.length;
      var extraBytes = len2 % 3;
      var parts = [];
      var maxChunkLength = 16383;
      for (var i2 = 0, len22 = len2 - extraBytes; i2 < len22; i2 += maxChunkLength) {
        parts.push(encodeChunk(uint8, i2, i2 + maxChunkLength > len22 ? len22 : i2 + maxChunkLength));
      }
      if (extraBytes === 1) {
        tmp = uint8[len2 - 1];
        parts.push(
          lookup[tmp >> 2] + lookup[tmp << 4 & 63] + "=="
        );
      } else if (extraBytes === 2) {
        tmp = (uint8[len2 - 2] << 8) + uint8[len2 - 1];
        parts.push(
          lookup[tmp >> 10] + lookup[tmp >> 4 & 63] + lookup[tmp << 2 & 63] + "="
        );
      }
      return parts.join("");
    }
  }
});

// node_modules/spacetimedb/dist/index.browser.mjs
var import_base64_js = __toESM(require_base64_js(), 1);
var TimeDuration = class _TimeDuration {
  __time_duration_micros__;
  static MICROS_PER_MILLIS = 1000n;
  /**
   * Get the algebraic type representation of the {@link TimeDuration} type.
   * @returns The algebraic type representation of the type.
   */
  static getAlgebraicType() {
    return AlgebraicType.Product({
      elements: [
        {
          name: "__time_duration_micros__",
          algebraicType: AlgebraicType.I64
        }
      ]
    });
  }
  get micros() {
    return this.__time_duration_micros__;
  }
  get millis() {
    return Number(this.micros / _TimeDuration.MICROS_PER_MILLIS);
  }
  constructor(micros) {
    this.__time_duration_micros__ = micros;
  }
  static fromMillis(millis) {
    return new _TimeDuration(BigInt(millis) * _TimeDuration.MICROS_PER_MILLIS);
  }
  /** This outputs the same string format that we use in the host and in Rust modules */
  toString() {
    const micros = this.micros;
    const sign = micros < 0 ? "-" : "+";
    const pos = micros < 0 ? -micros : micros;
    const secs = pos / 1000000n;
    const micros_remaining = pos % 1000000n;
    return `${sign}${secs}.${String(micros_remaining).padStart(6, "0")}`;
  }
};
var Timestamp = class _Timestamp {
  __timestamp_micros_since_unix_epoch__;
  static MICROS_PER_MILLIS = 1000n;
  get microsSinceUnixEpoch() {
    return this.__timestamp_micros_since_unix_epoch__;
  }
  constructor(micros) {
    this.__timestamp_micros_since_unix_epoch__ = micros;
  }
  /**
   * Get the algebraic type representation of the {@link Timestamp} type.
   * @returns The algebraic type representation of the type.
   */
  static getAlgebraicType() {
    return AlgebraicType.Product({
      elements: [
        {
          name: "__timestamp_micros_since_unix_epoch__",
          algebraicType: AlgebraicType.I64
        }
      ]
    });
  }
  /**
   * The Unix epoch, the midnight at the beginning of January 1, 1970, UTC.
   */
  static UNIX_EPOCH = new _Timestamp(0n);
  /**
   * Get a `Timestamp` representing the execution environment's belief of the current moment in time.
   */
  static now() {
    return _Timestamp.fromDate(/* @__PURE__ */ new Date());
  }
  /**
   * Get a `Timestamp` representing the same point in time as `date`.
   */
  static fromDate(date) {
    const millis = date.getTime();
    const micros = BigInt(millis) * _Timestamp.MICROS_PER_MILLIS;
    return new _Timestamp(micros);
  }
  /**
   * Get a `Date` representing approximately the same point in time as `this`.
   *
   * This method truncates to millisecond precision,
   * and throws `RangeError` if the `Timestamp` is outside the range representable as a `Date`.
   */
  toDate() {
    const micros = this.__timestamp_micros_since_unix_epoch__;
    const millis = micros / _Timestamp.MICROS_PER_MILLIS;
    if (millis > BigInt(Number.MAX_SAFE_INTEGER) || millis < BigInt(Number.MIN_SAFE_INTEGER)) {
      throw new RangeError(
        "Timestamp is outside of the representable range of JS's Date"
      );
    }
    return new Date(Number(millis));
  }
  since(other) {
    return new TimeDuration(
      this.__timestamp_micros_since_unix_epoch__ - other.__timestamp_micros_since_unix_epoch__
    );
  }
};
var BinaryWriter = class {
  #buffer;
  #view;
  #offset = 0;
  constructor(size) {
    this.#buffer = new Uint8Array(size);
    this.#view = new DataView(this.#buffer.buffer);
  }
  #expandBuffer(additionalCapacity) {
    const minCapacity = this.#offset + additionalCapacity + 1;
    if (minCapacity <= this.#buffer.length) return;
    let newCapacity = this.#buffer.length * 2;
    if (newCapacity < minCapacity) newCapacity = minCapacity;
    const newBuffer = new Uint8Array(newCapacity);
    newBuffer.set(this.#buffer);
    this.#buffer = newBuffer;
    this.#view = new DataView(this.#buffer.buffer);
  }
  toBase64() {
    return (0, import_base64_js.fromByteArray)(this.#buffer.subarray(0, this.#offset));
  }
  getBuffer() {
    return this.#buffer.slice(0, this.#offset);
  }
  get offset() {
    return this.#offset;
  }
  writeUInt8Array(value) {
    const length = value.length;
    this.#expandBuffer(4 + length);
    this.writeU32(length);
    this.#buffer.set(value, this.#offset);
    this.#offset += value.length;
  }
  writeBool(value) {
    this.#expandBuffer(1);
    this.#view.setUint8(this.#offset, value ? 1 : 0);
    this.#offset += 1;
  }
  writeByte(value) {
    this.#expandBuffer(1);
    this.#view.setUint8(this.#offset, value);
    this.#offset += 1;
  }
  writeI8(value) {
    this.#expandBuffer(1);
    this.#view.setInt8(this.#offset, value);
    this.#offset += 1;
  }
  writeU8(value) {
    this.#expandBuffer(1);
    this.#view.setUint8(this.#offset, value);
    this.#offset += 1;
  }
  writeI16(value) {
    this.#expandBuffer(2);
    this.#view.setInt16(this.#offset, value, true);
    this.#offset += 2;
  }
  writeU16(value) {
    this.#expandBuffer(2);
    this.#view.setUint16(this.#offset, value, true);
    this.#offset += 2;
  }
  writeI32(value) {
    this.#expandBuffer(4);
    this.#view.setInt32(this.#offset, value, true);
    this.#offset += 4;
  }
  writeU32(value) {
    this.#expandBuffer(4);
    this.#view.setUint32(this.#offset, value, true);
    this.#offset += 4;
  }
  writeI64(value) {
    this.#expandBuffer(8);
    this.#view.setBigInt64(this.#offset, value, true);
    this.#offset += 8;
  }
  writeU64(value) {
    this.#expandBuffer(8);
    this.#view.setBigUint64(this.#offset, value, true);
    this.#offset += 8;
  }
  writeU128(value) {
    this.#expandBuffer(16);
    const lowerPart = value & BigInt("0xFFFFFFFFFFFFFFFF");
    const upperPart = value >> BigInt(64);
    this.#view.setBigUint64(this.#offset, lowerPart, true);
    this.#view.setBigUint64(this.#offset + 8, upperPart, true);
    this.#offset += 16;
  }
  writeI128(value) {
    this.#expandBuffer(16);
    const lowerPart = value & BigInt("0xFFFFFFFFFFFFFFFF");
    const upperPart = value >> BigInt(64);
    this.#view.setBigInt64(this.#offset, lowerPart, true);
    this.#view.setBigInt64(this.#offset + 8, upperPart, true);
    this.#offset += 16;
  }
  writeU256(value) {
    this.#expandBuffer(32);
    const low_64_mask = BigInt("0xFFFFFFFFFFFFFFFF");
    const p0 = value & low_64_mask;
    const p1 = value >> BigInt(64 * 1) & low_64_mask;
    const p2 = value >> BigInt(64 * 2) & low_64_mask;
    const p3 = value >> BigInt(64 * 3);
    this.#view.setBigUint64(this.#offset + 8 * 0, p0, true);
    this.#view.setBigUint64(this.#offset + 8 * 1, p1, true);
    this.#view.setBigUint64(this.#offset + 8 * 2, p2, true);
    this.#view.setBigUint64(this.#offset + 8 * 3, p3, true);
    this.#offset += 32;
  }
  writeI256(value) {
    this.#expandBuffer(32);
    const low_64_mask = BigInt("0xFFFFFFFFFFFFFFFF");
    const p0 = value & low_64_mask;
    const p1 = value >> BigInt(64 * 1) & low_64_mask;
    const p2 = value >> BigInt(64 * 2) & low_64_mask;
    const p3 = value >> BigInt(64 * 3);
    this.#view.setBigUint64(this.#offset + 8 * 0, p0, true);
    this.#view.setBigUint64(this.#offset + 8 * 1, p1, true);
    this.#view.setBigUint64(this.#offset + 8 * 2, p2, true);
    this.#view.setBigInt64(this.#offset + 8 * 3, p3, true);
    this.#offset += 32;
  }
  writeF32(value) {
    this.#expandBuffer(4);
    this.#view.setFloat32(this.#offset, value, true);
    this.#offset += 4;
  }
  writeF64(value) {
    this.#expandBuffer(8);
    this.#view.setFloat64(this.#offset, value, true);
    this.#offset += 8;
  }
  writeString(value) {
    const encoder = new TextEncoder();
    const encodedString = encoder.encode(value);
    this.writeU32(encodedString.length);
    this.#expandBuffer(encodedString.length);
    this.#buffer.set(encodedString, this.#offset);
    this.#offset += encodedString.length;
  }
};
var BinaryReader = class {
  /**
   * The DataView used to read values from the binary data.
   *
   * Note: The DataView's `byteOffset` is relative to the beginning of the
   * underlying ArrayBuffer, not the start of the provided Uint8Array input.
   * This `BinaryReader`'s `#offset` field is used to track the current read position
   * relative to the start of the provided Uint8Array input.
   */
  #view;
  /**
   * Represents the offset (in bytes) relative to the start of the DataView
   * and provided Uint8Array input.
   *
   * Note: This is *not* the absolute byte offset within the underlying ArrayBuffer.
   */
  #offset = 0;
  constructor(input) {
    this.#view = new DataView(input.buffer, input.byteOffset, input.byteLength);
    this.#offset = 0;
  }
  get offset() {
    return this.#offset;
  }
  get remaining() {
    return this.#view.byteLength - this.#offset;
  }
  /** Ensure we have at least `n` bytes left to read */
  #ensure(n) {
    if (this.#offset + n > this.#view.byteLength) {
      throw new RangeError(
        `Tried to read ${n} byte(s) at relative offset ${this.#offset}, but only ${this.remaining} byte(s) remain`
      );
    }
  }
  readUInt8Array() {
    const length = this.readU32();
    this.#ensure(length);
    return this.readBytes(length);
  }
  readBool() {
    const value = this.#view.getUint8(this.#offset);
    this.#offset += 1;
    return value !== 0;
  }
  readByte() {
    const value = this.#view.getUint8(this.#offset);
    this.#offset += 1;
    return value;
  }
  readBytes(length) {
    const array = new Uint8Array(
      this.#view.buffer,
      this.#view.byteOffset + this.#offset,
      length
    );
    this.#offset += length;
    return array;
  }
  readI8() {
    const value = this.#view.getInt8(this.#offset);
    this.#offset += 1;
    return value;
  }
  readU8() {
    return this.readByte();
  }
  readI16() {
    const value = this.#view.getInt16(this.#offset, true);
    this.#offset += 2;
    return value;
  }
  readU16() {
    const value = this.#view.getUint16(this.#offset, true);
    this.#offset += 2;
    return value;
  }
  readI32() {
    const value = this.#view.getInt32(this.#offset, true);
    this.#offset += 4;
    return value;
  }
  readU32() {
    const value = this.#view.getUint32(this.#offset, true);
    this.#offset += 4;
    return value;
  }
  readI64() {
    const value = this.#view.getBigInt64(this.#offset, true);
    this.#offset += 8;
    return value;
  }
  readU64() {
    const value = this.#view.getBigUint64(this.#offset, true);
    this.#offset += 8;
    return value;
  }
  readU128() {
    const lowerPart = this.#view.getBigUint64(this.#offset, true);
    const upperPart = this.#view.getBigUint64(this.#offset + 8, true);
    this.#offset += 16;
    return (upperPart << BigInt(64)) + lowerPart;
  }
  readI128() {
    const lowerPart = this.#view.getBigUint64(this.#offset, true);
    const upperPart = this.#view.getBigInt64(this.#offset + 8, true);
    this.#offset += 16;
    return (upperPart << BigInt(64)) + lowerPart;
  }
  readU256() {
    const p0 = this.#view.getBigUint64(this.#offset, true);
    const p1 = this.#view.getBigUint64(this.#offset + 8, true);
    const p2 = this.#view.getBigUint64(this.#offset + 16, true);
    const p3 = this.#view.getBigUint64(this.#offset + 24, true);
    this.#offset += 32;
    return (p3 << BigInt(3 * 64)) + (p2 << BigInt(2 * 64)) + (p1 << BigInt(1 * 64)) + p0;
  }
  readI256() {
    const p0 = this.#view.getBigUint64(this.#offset, true);
    const p1 = this.#view.getBigUint64(this.#offset + 8, true);
    const p2 = this.#view.getBigUint64(this.#offset + 16, true);
    const p3 = this.#view.getBigInt64(this.#offset + 24, true);
    this.#offset += 32;
    return (p3 << BigInt(3 * 64)) + (p2 << BigInt(2 * 64)) + (p1 << BigInt(1 * 64)) + p0;
  }
  readF32() {
    const value = this.#view.getFloat32(this.#offset, true);
    this.#offset += 4;
    return value;
  }
  readF64() {
    const value = this.#view.getFloat64(this.#offset, true);
    this.#offset += 8;
    return value;
  }
  readString() {
    const uint8Array = this.readUInt8Array();
    return new TextDecoder("utf-8").decode(uint8Array);
  }
};
function deepEqual(obj1, obj2) {
  if (obj1 === obj2) return true;
  if (typeof obj1 !== "object" || obj1 === null || typeof obj2 !== "object" || obj2 === null) {
    return false;
  }
  const keys1 = Object.keys(obj1);
  const keys2 = Object.keys(obj2);
  if (keys1.length !== keys2.length) return false;
  for (const key of keys1) {
    if (!keys2.includes(key) || !deepEqual(obj1[key], obj2[key])) {
      return false;
    }
  }
  return true;
}
function uint8ArrayToHexString(array) {
  return Array.prototype.map.call(array.reverse(), (x) => ("00" + x.toString(16)).slice(-2)).join("");
}
function uint8ArrayToU128(array) {
  if (array.length != 16) {
    throw new Error(`Uint8Array is not 16 bytes long: ${array}`);
  }
  return new BinaryReader(array).readU128();
}
function uint8ArrayToU256(array) {
  if (array.length != 32) {
    throw new Error(`Uint8Array is not 32 bytes long: [${array}]`);
  }
  return new BinaryReader(array).readU256();
}
function hexStringToUint8Array(str) {
  if (str.startsWith("0x")) {
    str = str.slice(2);
  }
  const matches = str.match(/.{1,2}/g) || [];
  const data = Uint8Array.from(
    matches.map((byte) => parseInt(byte, 16))
  );
  return data.reverse();
}
function hexStringToU128(str) {
  return uint8ArrayToU128(hexStringToUint8Array(str));
}
function hexStringToU256(str) {
  return uint8ArrayToU256(hexStringToUint8Array(str));
}
function u128ToUint8Array(data) {
  const writer = new BinaryWriter(16);
  writer.writeU128(data);
  return writer.getBuffer();
}
function u128ToHexString(data) {
  return uint8ArrayToHexString(u128ToUint8Array(data));
}
function u256ToUint8Array(data) {
  const writer = new BinaryWriter(32);
  writer.writeU256(data);
  return writer.getBuffer();
}
function u256ToHexString(data) {
  return uint8ArrayToHexString(u256ToUint8Array(data));
}
var Identity = class _Identity {
  __identity__;
  /**
   * Creates a new `Identity`.
   *
   * `data` can be a hexadecimal string or a `bigint`.
   */
  constructor(data) {
    this.__identity__ = typeof data === "string" ? hexStringToU256(data) : data;
  }
  /**
   * Get the algebraic type representation of the {@link Identity} type.
   * @returns The algebraic type representation of the type.
   */
  static getAlgebraicType() {
    return AlgebraicType.Product({
      elements: [{ name: "__identity__", algebraicType: AlgebraicType.U256 }]
    });
  }
  /**
   * Compare two identities for equality.
   */
  isEqual(other) {
    return this.toHexString() === other.toHexString();
  }
  /**
   * Print the identity as a hexadecimal string.
   */
  toHexString() {
    return u256ToHexString(this.__identity__);
  }
  /**
   * Convert the address to a Uint8Array.
   */
  toUint8Array() {
    return u256ToUint8Array(this.__identity__);
  }
  /**
   * Parse an Identity from a hexadecimal string.
   */
  static fromString(str) {
    return new _Identity(str);
  }
  /**
   * Zero identity (0x0000000000000000000000000000000000000000000000000000000000000000)
   */
  static zero() {
    return new _Identity(0n);
  }
  toString() {
    return this.toHexString();
  }
};
var Option = {
  getAlgebraicType(innerType) {
    return AlgebraicType.Sum({
      variants: [
        { name: "some", algebraicType: innerType },
        {
          name: "none",
          algebraicType: AlgebraicType.Product({ elements: [] })
        }
      ]
    });
  }
};
var _cached_SumTypeVariant_type_value = null;
var SumTypeVariant = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_SumTypeVariant_type_value)
      return _cached_SumTypeVariant_type_value;
    _cached_SumTypeVariant_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_SumTypeVariant_type_value.value.elements.push(
      {
        name: "name",
        algebraicType: AlgebraicType.createOptionType(
          AlgebraicType.String
        )
      },
      {
        name: "algebraicType",
        algebraicType: AlgebraicType2.getTypeScriptAlgebraicType()
      }
    );
    return _cached_SumTypeVariant_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      SumTypeVariant.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      SumTypeVariant.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_SumType_type_value = null;
var SumType = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_SumType_type_value) return _cached_SumType_type_value;
    _cached_SumType_type_value = AlgebraicType.Product({ elements: [] });
    _cached_SumType_type_value.value.elements.push({
      name: "variants",
      algebraicType: AlgebraicType.Array(
        SumTypeVariant.getTypeScriptAlgebraicType()
      )
    });
    return _cached_SumType_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      SumType.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      SumType.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_ProductTypeElement_type_value = null;
var ProductTypeElement = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_ProductTypeElement_type_value)
      return _cached_ProductTypeElement_type_value;
    _cached_ProductTypeElement_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_ProductTypeElement_type_value.value.elements.push(
      {
        name: "name",
        algebraicType: AlgebraicType.createOptionType(
          AlgebraicType.String
        )
      },
      {
        name: "algebraicType",
        algebraicType: AlgebraicType2.getTypeScriptAlgebraicType()
      }
    );
    return _cached_ProductTypeElement_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      ProductTypeElement.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      ProductTypeElement.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_ProductType_type_value = null;
var ProductType = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_ProductType_type_value) return _cached_ProductType_type_value;
    _cached_ProductType_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_ProductType_type_value.value.elements.push({
      name: "elements",
      algebraicType: AlgebraicType.Array(
        ProductTypeElement.getTypeScriptAlgebraicType()
      )
    });
    return _cached_ProductType_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      ProductType.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      ProductType.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_AlgebraicType_type_value = null;
var AlgebraicType2 = {
  // Helper functions for constructing each variant of the tagged union.
  // ```
  // const foo = Foo.A(42);
  // assert!(foo.tag === "A");
  // assert!(foo.value === 42);
  // ```
  Ref: (value) => ({ tag: "Ref", value }),
  Sum: (value) => ({ tag: "Sum", value }),
  Product: (value) => ({
    tag: "Product",
    value
  }),
  Array: (value) => ({
    tag: "Array",
    value
  }),
  String: { tag: "String" },
  Bool: { tag: "Bool" },
  I8: { tag: "I8" },
  U8: { tag: "U8" },
  I16: { tag: "I16" },
  U16: { tag: "U16" },
  I32: { tag: "I32" },
  U32: { tag: "U32" },
  I64: { tag: "I64" },
  U64: { tag: "U64" },
  I128: { tag: "I128" },
  U128: { tag: "U128" },
  I256: { tag: "I256" },
  U256: { tag: "U256" },
  F32: { tag: "F32" },
  F64: { tag: "F64" },
  getTypeScriptAlgebraicType() {
    if (_cached_AlgebraicType_type_value)
      return _cached_AlgebraicType_type_value;
    _cached_AlgebraicType_type_value = AlgebraicType.Sum({
      variants: []
    });
    _cached_AlgebraicType_type_value.value.variants.push(
      { name: "Ref", algebraicType: AlgebraicType.U32 },
      { name: "Sum", algebraicType: SumType.getTypeScriptAlgebraicType() },
      {
        name: "Product",
        algebraicType: ProductType.getTypeScriptAlgebraicType()
      },
      {
        name: "Array",
        algebraicType: AlgebraicType2.getTypeScriptAlgebraicType()
      },
      {
        name: "String",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "Bool",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "I8",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "U8",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "I16",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "U16",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "I32",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "U32",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "I64",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "U64",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "I128",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "U128",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "I256",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "U256",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "F32",
        algebraicType: AlgebraicType.Product({ elements: [] })
      },
      {
        name: "F64",
        algebraicType: AlgebraicType.Product({ elements: [] })
      }
    );
    return _cached_AlgebraicType_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      AlgebraicType2.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      AlgebraicType2.getTypeScriptAlgebraicType()
    );
  }
};
var ScheduleAt = {
  interval(value) {
    return Interval(value);
  },
  time(value) {
    return Time(value);
  },
  getAlgebraicType() {
    return AlgebraicType.Sum({
      variants: [
        {
          name: "Interval",
          algebraicType: TimeDuration.getAlgebraicType()
        },
        { name: "Time", algebraicType: Timestamp.getAlgebraicType() }
      ]
    });
  }
};
var Interval = (micros) => ({
  tag: "Interval",
  value: new TimeDuration(micros)
});
var Time = (microsSinceUnixEpoch) => ({
  tag: "Time",
  value: new Timestamp(microsSinceUnixEpoch)
});
var schedule_at_default = ScheduleAt;
var AlgebraicType = {
  ...AlgebraicType2,
  Sum: (value) => ({
    tag: "Sum",
    value
  }),
  Product: (value) => ({
    tag: "Product",
    value
  }),
  Array: (value) => ({
    tag: "Array",
    value
  }),
  createOptionType: function(innerType) {
    return Option.getAlgebraicType(innerType);
  },
  createIdentityType: function() {
    return Identity.getAlgebraicType();
  },
  createConnectionIdType: function() {
    return ConnectionId.getAlgebraicType();
  },
  createScheduleAtType: function() {
    return schedule_at_default.getAlgebraicType();
  },
  createTimestampType: function() {
    return Timestamp.getAlgebraicType();
  },
  createTimeDurationType: function() {
    return TimeDuration.getAlgebraicType();
  },
  serializeValue: function(writer, ty, value, typespace) {
    if (ty.tag === "Ref") {
      if (!typespace)
        throw new Error("cannot serialize refs without a typespace");
      while (ty.tag === "Ref") ty = typespace.types[ty.value];
    }
    switch (ty.tag) {
      case "Product":
        ProductType2.serializeValue(writer, ty.value, value, typespace);
        break;
      case "Sum":
        SumType2.serializeValue(writer, ty.value, value, typespace);
        break;
      case "Array":
        if (ty.value.tag === "U8") {
          writer.writeUInt8Array(value);
        } else {
          const elemType = ty.value;
          writer.writeU32(value.length);
          for (const elem of value) {
            AlgebraicType.serializeValue(writer, elemType, elem, typespace);
          }
        }
        break;
      case "Bool":
        writer.writeBool(value);
        break;
      case "I8":
        writer.writeI8(value);
        break;
      case "U8":
        writer.writeU8(value);
        break;
      case "I16":
        writer.writeI16(value);
        break;
      case "U16":
        writer.writeU16(value);
        break;
      case "I32":
        writer.writeI32(value);
        break;
      case "U32":
        writer.writeU32(value);
        break;
      case "I64":
        writer.writeI64(value);
        break;
      case "U64":
        writer.writeU64(value);
        break;
      case "I128":
        writer.writeI128(value);
        break;
      case "U128":
        writer.writeU128(value);
        break;
      case "I256":
        writer.writeI256(value);
        break;
      case "U256":
        writer.writeU256(value);
        break;
      case "F32":
        writer.writeF32(value);
        break;
      case "F64":
        writer.writeF64(value);
        break;
      case "String":
        writer.writeString(value);
        break;
    }
  },
  deserializeValue: function(reader, ty, typespace) {
    if (ty.tag === "Ref") {
      if (!typespace)
        throw new Error("cannot deserialize refs without a typespace");
      while (ty.tag === "Ref") ty = typespace.types[ty.value];
    }
    switch (ty.tag) {
      case "Product":
        return ProductType2.deserializeValue(reader, ty.value, typespace);
      case "Sum":
        return SumType2.deserializeValue(reader, ty.value, typespace);
      case "Array":
        if (ty.value.tag === "U8") {
          return reader.readUInt8Array();
        } else {
          const elemType = ty.value;
          const length = reader.readU32();
          const result = [];
          for (let i = 0; i < length; i++) {
            result.push(
              AlgebraicType.deserializeValue(reader, elemType, typespace)
            );
          }
          return result;
        }
      case "Bool":
        return reader.readBool();
      case "I8":
        return reader.readI8();
      case "U8":
        return reader.readU8();
      case "I16":
        return reader.readI16();
      case "U16":
        return reader.readU16();
      case "I32":
        return reader.readI32();
      case "U32":
        return reader.readU32();
      case "I64":
        return reader.readI64();
      case "U64":
        return reader.readU64();
      case "I128":
        return reader.readI128();
      case "U128":
        return reader.readU128();
      case "I256":
        return reader.readI256();
      case "U256":
        return reader.readU256();
      case "F32":
        return reader.readF32();
      case "F64":
        return reader.readF64();
      case "String":
        return reader.readString();
    }
  },
  /**
   * Convert a value of the algebraic type into something that can be used as a key in a map.
   * There are no guarantees about being able to order it.
   * This is only guaranteed to be comparable to other values of the same type.
   * @param value A value of the algebraic type
   * @returns Something that can be used as a key in a map.
   */
  intoMapKey: function(ty, value) {
    switch (ty.tag) {
      case "U8":
      case "U16":
      case "U32":
      case "U64":
      case "U128":
      case "U256":
      case "I8":
      case "I16":
      case "I32":
      case "I64":
      case "I128":
      case "I256":
      case "F32":
      case "F64":
      case "String":
      case "Bool":
        return value;
      case "Product":
        return ProductType2.intoMapKey(ty.value, value);
      default: {
        const writer = new BinaryWriter(10);
        AlgebraicType.serializeValue(writer, ty, value);
        return writer.toBase64();
      }
    }
  }
};
var ProductType2 = {
  ...ProductType,
  serializeValue(writer, ty, value, typespace) {
    for (const element of ty.elements) {
      AlgebraicType.serializeValue(
        writer,
        element.algebraicType,
        value[element.name],
        typespace
      );
    }
  },
  deserializeValue(reader, ty, typespace) {
    const result = {};
    if (ty.elements.length === 1) {
      if (ty.elements[0].name === "__time_duration_micros__") {
        return new TimeDuration(reader.readI64());
      }
      if (ty.elements[0].name === "__timestamp_micros_since_unix_epoch__") {
        return new Timestamp(reader.readI64());
      }
      if (ty.elements[0].name === "__identity__") {
        return new Identity(reader.readU256());
      }
      if (ty.elements[0].name === "__connection_id__") {
        return new ConnectionId(reader.readU128());
      }
    }
    for (const element of ty.elements) {
      result[element.name] = AlgebraicType.deserializeValue(
        reader,
        element.algebraicType,
        typespace
      );
    }
    return result;
  },
  intoMapKey(ty, value) {
    if (ty.elements.length === 1) {
      if (ty.elements[0].name === "__time_duration_micros__") {
        return value.__time_duration_micros__;
      }
      if (ty.elements[0].name === "__timestamp_micros_since_unix_epoch__") {
        return value.__timestamp_micros_since_unix_epoch__;
      }
      if (ty.elements[0].name === "__identity__") {
        return value.__identity__;
      }
      if (ty.elements[0].name === "__connection_id__") {
        return value.__connection_id__;
      }
    }
    const writer = new BinaryWriter(10);
    AlgebraicType.serializeValue(writer, AlgebraicType.Product(ty), value);
    return writer.toBase64();
  }
};
var SumType2 = {
  ...SumType,
  serializeValue: function(writer, ty, value, typespace) {
    if (ty.variants.length == 2 && ty.variants[0].name === "some" && ty.variants[1].name === "none") {
      if (value !== null && value !== void 0) {
        writer.writeByte(0);
        AlgebraicType.serializeValue(
          writer,
          ty.variants[0].algebraicType,
          value,
          typespace
        );
      } else {
        writer.writeByte(1);
      }
    } else {
      const variant = value["tag"];
      const index = ty.variants.findIndex((v) => v.name === variant);
      if (index < 0) {
        throw `Can't serialize a sum type, couldn't find ${value.tag} tag`;
      }
      writer.writeU8(index);
      AlgebraicType.serializeValue(
        writer,
        ty.variants[index].algebraicType,
        value["value"],
        typespace
      );
    }
  },
  deserializeValue: function(reader, ty, typespace) {
    const tag = reader.readU8();
    if (ty.variants.length == 2 && ty.variants[0].name === "some" && ty.variants[1].name === "none") {
      if (tag === 0) {
        return AlgebraicType.deserializeValue(
          reader,
          ty.variants[0].algebraicType,
          typespace
        );
      } else if (tag === 1) {
        return void 0;
      } else {
        throw `Can't deserialize an option type, couldn't find ${tag} tag`;
      }
    } else {
      const variant = ty.variants[tag];
      const value = AlgebraicType.deserializeValue(
        reader,
        variant.algebraicType,
        typespace
      );
      return { tag: variant.name, value };
    }
  }
};
var ConnectionId = class _ConnectionId {
  __connection_id__;
  /**
   * Creates a new `ConnectionId`.
   */
  constructor(data) {
    this.__connection_id__ = data;
  }
  /**
   * Get the algebraic type representation of the {@link ConnectionId} type.
   * @returns The algebraic type representation of the type.
   */
  static getAlgebraicType() {
    return AlgebraicType.Product({
      elements: [
        { name: "__connection_id__", algebraicType: AlgebraicType.U128 }
      ]
    });
  }
  isZero() {
    return this.__connection_id__ === BigInt(0);
  }
  static nullIfZero(addr) {
    if (addr.isZero()) {
      return null;
    } else {
      return addr;
    }
  }
  static random() {
    function randomU8() {
      return Math.floor(Math.random() * 255);
    }
    let result = BigInt(0);
    for (let i = 0; i < 16; i++) {
      result = result << BigInt(8) | BigInt(randomU8());
    }
    return new _ConnectionId(result);
  }
  /**
   * Compare two connection IDs for equality.
   */
  isEqual(other) {
    return this.__connection_id__ == other.__connection_id__;
  }
  /**
   * Print the connection ID as a hexadecimal string.
   */
  toHexString() {
    return u128ToHexString(this.__connection_id__);
  }
  /**
   * Convert the connection ID to a Uint8Array.
   */
  toUint8Array() {
    return u128ToUint8Array(this.__connection_id__);
  }
  /**
   * Parse a connection ID from a hexadecimal string.
   */
  static fromString(str) {
    return new _ConnectionId(hexStringToU128(str));
  }
  static fromStringOrNull(str) {
    const addr = _ConnectionId.fromString(str);
    if (addr.isZero()) {
      return null;
    } else {
      return addr;
    }
  }
};
function parseValue(ty, src) {
  const reader = new BinaryReader(src);
  return ty.deserialize(reader);
}
var _cached_RowSizeHint_type_value = null;
var RowSizeHint = {
  // Helper functions for constructing each variant of the tagged union.
  // ```
  // const foo = Foo.A(42);
  // assert!(foo.tag === "A");
  // assert!(foo.value === 42);
  // ```
  FixedSize: (value) => ({
    tag: "FixedSize",
    value
  }),
  RowOffsets: (value) => ({
    tag: "RowOffsets",
    value
  }),
  getTypeScriptAlgebraicType() {
    if (_cached_RowSizeHint_type_value) return _cached_RowSizeHint_type_value;
    _cached_RowSizeHint_type_value = AlgebraicType.Sum({ variants: [] });
    _cached_RowSizeHint_type_value.value.variants.push(
      { name: "FixedSize", algebraicType: AlgebraicType.U16 },
      {
        name: "RowOffsets",
        algebraicType: AlgebraicType.Array(AlgebraicType.U64)
      }
    );
    return _cached_RowSizeHint_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      RowSizeHint.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      RowSizeHint.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_BsatnRowList_type_value = null;
var BsatnRowList = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_BsatnRowList_type_value) return _cached_BsatnRowList_type_value;
    _cached_BsatnRowList_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_BsatnRowList_type_value.value.elements.push(
      {
        name: "sizeHint",
        algebraicType: RowSizeHint.getTypeScriptAlgebraicType()
      },
      {
        name: "rowsData",
        algebraicType: AlgebraicType.Array(AlgebraicType.U8)
      }
    );
    return _cached_BsatnRowList_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      BsatnRowList.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      BsatnRowList.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_CallReducer_type_value = null;
var CallReducer = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_CallReducer_type_value) return _cached_CallReducer_type_value;
    _cached_CallReducer_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_CallReducer_type_value.value.elements.push(
      { name: "reducer", algebraicType: AlgebraicType.String },
      {
        name: "args",
        algebraicType: AlgebraicType.Array(AlgebraicType.U8)
      },
      { name: "requestId", algebraicType: AlgebraicType.U32 },
      { name: "flags", algebraicType: AlgebraicType.U8 }
    );
    return _cached_CallReducer_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      CallReducer.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      CallReducer.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_Subscribe_type_value = null;
var Subscribe = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_Subscribe_type_value) return _cached_Subscribe_type_value;
    _cached_Subscribe_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_Subscribe_type_value.value.elements.push(
      {
        name: "queryStrings",
        algebraicType: AlgebraicType.Array(AlgebraicType.String)
      },
      { name: "requestId", algebraicType: AlgebraicType.U32 }
    );
    return _cached_Subscribe_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      Subscribe.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      Subscribe.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_OneOffQuery_type_value = null;
var OneOffQuery = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_OneOffQuery_type_value) return _cached_OneOffQuery_type_value;
    _cached_OneOffQuery_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_OneOffQuery_type_value.value.elements.push(
      {
        name: "messageId",
        algebraicType: AlgebraicType.Array(AlgebraicType.U8)
      },
      { name: "queryString", algebraicType: AlgebraicType.String }
    );
    return _cached_OneOffQuery_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      OneOffQuery.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      OneOffQuery.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_QueryId_type_value = null;
var QueryId = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_QueryId_type_value) return _cached_QueryId_type_value;
    _cached_QueryId_type_value = AlgebraicType.Product({ elements: [] });
    _cached_QueryId_type_value.value.elements.push({
      name: "id",
      algebraicType: AlgebraicType.U32
    });
    return _cached_QueryId_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      QueryId.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      QueryId.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_SubscribeSingle_type_value = null;
var SubscribeSingle = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_SubscribeSingle_type_value)
      return _cached_SubscribeSingle_type_value;
    _cached_SubscribeSingle_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_SubscribeSingle_type_value.value.elements.push(
      { name: "query", algebraicType: AlgebraicType.String },
      { name: "requestId", algebraicType: AlgebraicType.U32 },
      { name: "queryId", algebraicType: QueryId.getTypeScriptAlgebraicType() }
    );
    return _cached_SubscribeSingle_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      SubscribeSingle.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      SubscribeSingle.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_SubscribeMulti_type_value = null;
var SubscribeMulti = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_SubscribeMulti_type_value)
      return _cached_SubscribeMulti_type_value;
    _cached_SubscribeMulti_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_SubscribeMulti_type_value.value.elements.push(
      {
        name: "queryStrings",
        algebraicType: AlgebraicType.Array(AlgebraicType.String)
      },
      { name: "requestId", algebraicType: AlgebraicType.U32 },
      { name: "queryId", algebraicType: QueryId.getTypeScriptAlgebraicType() }
    );
    return _cached_SubscribeMulti_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      SubscribeMulti.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      SubscribeMulti.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_Unsubscribe_type_value = null;
var Unsubscribe = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_Unsubscribe_type_value) return _cached_Unsubscribe_type_value;
    _cached_Unsubscribe_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_Unsubscribe_type_value.value.elements.push(
      { name: "requestId", algebraicType: AlgebraicType.U32 },
      { name: "queryId", algebraicType: QueryId.getTypeScriptAlgebraicType() }
    );
    return _cached_Unsubscribe_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      Unsubscribe.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      Unsubscribe.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_UnsubscribeMulti_type_value = null;
var UnsubscribeMulti = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_UnsubscribeMulti_type_value)
      return _cached_UnsubscribeMulti_type_value;
    _cached_UnsubscribeMulti_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_UnsubscribeMulti_type_value.value.elements.push(
      { name: "requestId", algebraicType: AlgebraicType.U32 },
      { name: "queryId", algebraicType: QueryId.getTypeScriptAlgebraicType() }
    );
    return _cached_UnsubscribeMulti_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      UnsubscribeMulti.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      UnsubscribeMulti.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_ClientMessage_type_value = null;
var ClientMessage = {
  // Helper functions for constructing each variant of the tagged union.
  // ```
  // const foo = Foo.A(42);
  // assert!(foo.tag === "A");
  // assert!(foo.value === 42);
  // ```
  CallReducer: (value) => ({
    tag: "CallReducer",
    value
  }),
  Subscribe: (value) => ({
    tag: "Subscribe",
    value
  }),
  OneOffQuery: (value) => ({
    tag: "OneOffQuery",
    value
  }),
  SubscribeSingle: (value) => ({
    tag: "SubscribeSingle",
    value
  }),
  SubscribeMulti: (value) => ({ tag: "SubscribeMulti", value }),
  Unsubscribe: (value) => ({
    tag: "Unsubscribe",
    value
  }),
  UnsubscribeMulti: (value) => ({
    tag: "UnsubscribeMulti",
    value
  }),
  getTypeScriptAlgebraicType() {
    if (_cached_ClientMessage_type_value)
      return _cached_ClientMessage_type_value;
    _cached_ClientMessage_type_value = AlgebraicType.Sum({
      variants: []
    });
    _cached_ClientMessage_type_value.value.variants.push(
      {
        name: "CallReducer",
        algebraicType: CallReducer.getTypeScriptAlgebraicType()
      },
      {
        name: "Subscribe",
        algebraicType: Subscribe.getTypeScriptAlgebraicType()
      },
      {
        name: "OneOffQuery",
        algebraicType: OneOffQuery.getTypeScriptAlgebraicType()
      },
      {
        name: "SubscribeSingle",
        algebraicType: SubscribeSingle.getTypeScriptAlgebraicType()
      },
      {
        name: "SubscribeMulti",
        algebraicType: SubscribeMulti.getTypeScriptAlgebraicType()
      },
      {
        name: "Unsubscribe",
        algebraicType: Unsubscribe.getTypeScriptAlgebraicType()
      },
      {
        name: "UnsubscribeMulti",
        algebraicType: UnsubscribeMulti.getTypeScriptAlgebraicType()
      }
    );
    return _cached_ClientMessage_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      ClientMessage.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      ClientMessage.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_QueryUpdate_type_value = null;
var QueryUpdate = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_QueryUpdate_type_value) return _cached_QueryUpdate_type_value;
    _cached_QueryUpdate_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_QueryUpdate_type_value.value.elements.push(
      {
        name: "deletes",
        algebraicType: BsatnRowList.getTypeScriptAlgebraicType()
      },
      {
        name: "inserts",
        algebraicType: BsatnRowList.getTypeScriptAlgebraicType()
      }
    );
    return _cached_QueryUpdate_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      QueryUpdate.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      QueryUpdate.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_CompressableQueryUpdate_type_value = null;
var CompressableQueryUpdate = {
  // Helper functions for constructing each variant of the tagged union.
  // ```
  // const foo = Foo.A(42);
  // assert!(foo.tag === "A");
  // assert!(foo.value === 42);
  // ```
  Uncompressed: (value) => ({
    tag: "Uncompressed",
    value
  }),
  Brotli: (value) => ({
    tag: "Brotli",
    value
  }),
  Gzip: (value) => ({
    tag: "Gzip",
    value
  }),
  getTypeScriptAlgebraicType() {
    if (_cached_CompressableQueryUpdate_type_value)
      return _cached_CompressableQueryUpdate_type_value;
    _cached_CompressableQueryUpdate_type_value = AlgebraicType.Sum({
      variants: []
    });
    _cached_CompressableQueryUpdate_type_value.value.variants.push(
      {
        name: "Uncompressed",
        algebraicType: QueryUpdate.getTypeScriptAlgebraicType()
      },
      {
        name: "Brotli",
        algebraicType: AlgebraicType.Array(AlgebraicType.U8)
      },
      {
        name: "Gzip",
        algebraicType: AlgebraicType.Array(AlgebraicType.U8)
      }
    );
    return _cached_CompressableQueryUpdate_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      CompressableQueryUpdate.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      CompressableQueryUpdate.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_TableUpdate_type_value = null;
var TableUpdate = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_TableUpdate_type_value) return _cached_TableUpdate_type_value;
    _cached_TableUpdate_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_TableUpdate_type_value.value.elements.push(
      { name: "tableId", algebraicType: AlgebraicType.U32 },
      { name: "tableName", algebraicType: AlgebraicType.String },
      { name: "numRows", algebraicType: AlgebraicType.U64 },
      {
        name: "updates",
        algebraicType: AlgebraicType.Array(
          CompressableQueryUpdate.getTypeScriptAlgebraicType()
        )
      }
    );
    return _cached_TableUpdate_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      TableUpdate.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      TableUpdate.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_DatabaseUpdate_type_value = null;
var DatabaseUpdate = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_DatabaseUpdate_type_value)
      return _cached_DatabaseUpdate_type_value;
    _cached_DatabaseUpdate_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_DatabaseUpdate_type_value.value.elements.push({
      name: "tables",
      algebraicType: AlgebraicType.Array(
        TableUpdate.getTypeScriptAlgebraicType()
      )
    });
    return _cached_DatabaseUpdate_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      DatabaseUpdate.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      DatabaseUpdate.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_InitialSubscription_type_value = null;
var InitialSubscription = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_InitialSubscription_type_value)
      return _cached_InitialSubscription_type_value;
    _cached_InitialSubscription_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_InitialSubscription_type_value.value.elements.push(
      {
        name: "databaseUpdate",
        algebraicType: DatabaseUpdate.getTypeScriptAlgebraicType()
      },
      { name: "requestId", algebraicType: AlgebraicType.U32 },
      {
        name: "totalHostExecutionDuration",
        algebraicType: AlgebraicType.createTimeDurationType()
      }
    );
    return _cached_InitialSubscription_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      InitialSubscription.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      InitialSubscription.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_UpdateStatus_type_value = null;
var UpdateStatus = {
  // Helper functions for constructing each variant of the tagged union.
  // ```
  // const foo = Foo.A(42);
  // assert!(foo.tag === "A");
  // assert!(foo.value === 42);
  // ```
  Committed: (value) => ({
    tag: "Committed",
    value
  }),
  Failed: (value) => ({
    tag: "Failed",
    value
  }),
  OutOfEnergy: { tag: "OutOfEnergy" },
  getTypeScriptAlgebraicType() {
    if (_cached_UpdateStatus_type_value) return _cached_UpdateStatus_type_value;
    _cached_UpdateStatus_type_value = AlgebraicType.Sum({
      variants: []
    });
    _cached_UpdateStatus_type_value.value.variants.push(
      {
        name: "Committed",
        algebraicType: DatabaseUpdate.getTypeScriptAlgebraicType()
      },
      { name: "Failed", algebraicType: AlgebraicType.String },
      {
        name: "OutOfEnergy",
        algebraicType: AlgebraicType.Product({ elements: [] })
      }
    );
    return _cached_UpdateStatus_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      UpdateStatus.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      UpdateStatus.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_ReducerCallInfo_type_value = null;
var ReducerCallInfo = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_ReducerCallInfo_type_value)
      return _cached_ReducerCallInfo_type_value;
    _cached_ReducerCallInfo_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_ReducerCallInfo_type_value.value.elements.push(
      { name: "reducerName", algebraicType: AlgebraicType.String },
      { name: "reducerId", algebraicType: AlgebraicType.U32 },
      {
        name: "args",
        algebraicType: AlgebraicType.Array(AlgebraicType.U8)
      },
      { name: "requestId", algebraicType: AlgebraicType.U32 }
    );
    return _cached_ReducerCallInfo_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      ReducerCallInfo.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      ReducerCallInfo.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_EnergyQuanta_type_value = null;
var EnergyQuanta = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_EnergyQuanta_type_value) return _cached_EnergyQuanta_type_value;
    _cached_EnergyQuanta_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_EnergyQuanta_type_value.value.elements.push({
      name: "quanta",
      algebraicType: AlgebraicType.U128
    });
    return _cached_EnergyQuanta_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      EnergyQuanta.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      EnergyQuanta.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_TransactionUpdate_type_value = null;
var TransactionUpdate = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_TransactionUpdate_type_value)
      return _cached_TransactionUpdate_type_value;
    _cached_TransactionUpdate_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_TransactionUpdate_type_value.value.elements.push(
      {
        name: "status",
        algebraicType: UpdateStatus.getTypeScriptAlgebraicType()
      },
      {
        name: "timestamp",
        algebraicType: AlgebraicType.createTimestampType()
      },
      {
        name: "callerIdentity",
        algebraicType: AlgebraicType.createIdentityType()
      },
      {
        name: "callerConnectionId",
        algebraicType: AlgebraicType.createConnectionIdType()
      },
      {
        name: "reducerCall",
        algebraicType: ReducerCallInfo.getTypeScriptAlgebraicType()
      },
      {
        name: "energyQuantaUsed",
        algebraicType: EnergyQuanta.getTypeScriptAlgebraicType()
      },
      {
        name: "totalHostExecutionDuration",
        algebraicType: AlgebraicType.createTimeDurationType()
      }
    );
    return _cached_TransactionUpdate_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      TransactionUpdate.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      TransactionUpdate.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_TransactionUpdateLight_type_value = null;
var TransactionUpdateLight = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_TransactionUpdateLight_type_value)
      return _cached_TransactionUpdateLight_type_value;
    _cached_TransactionUpdateLight_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_TransactionUpdateLight_type_value.value.elements.push(
      { name: "requestId", algebraicType: AlgebraicType.U32 },
      {
        name: "update",
        algebraicType: DatabaseUpdate.getTypeScriptAlgebraicType()
      }
    );
    return _cached_TransactionUpdateLight_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      TransactionUpdateLight.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      TransactionUpdateLight.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_IdentityToken_type_value = null;
var IdentityToken = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_IdentityToken_type_value)
      return _cached_IdentityToken_type_value;
    _cached_IdentityToken_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_IdentityToken_type_value.value.elements.push(
      {
        name: "identity",
        algebraicType: AlgebraicType.createIdentityType()
      },
      { name: "token", algebraicType: AlgebraicType.String },
      {
        name: "connectionId",
        algebraicType: AlgebraicType.createConnectionIdType()
      }
    );
    return _cached_IdentityToken_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      IdentityToken.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      IdentityToken.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_OneOffTable_type_value = null;
var OneOffTable = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_OneOffTable_type_value) return _cached_OneOffTable_type_value;
    _cached_OneOffTable_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_OneOffTable_type_value.value.elements.push(
      { name: "tableName", algebraicType: AlgebraicType.String },
      { name: "rows", algebraicType: BsatnRowList.getTypeScriptAlgebraicType() }
    );
    return _cached_OneOffTable_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      OneOffTable.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      OneOffTable.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_OneOffQueryResponse_type_value = null;
var OneOffQueryResponse = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_OneOffQueryResponse_type_value)
      return _cached_OneOffQueryResponse_type_value;
    _cached_OneOffQueryResponse_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_OneOffQueryResponse_type_value.value.elements.push(
      {
        name: "messageId",
        algebraicType: AlgebraicType.Array(AlgebraicType.U8)
      },
      {
        name: "error",
        algebraicType: AlgebraicType.createOptionType(
          AlgebraicType.String
        )
      },
      {
        name: "tables",
        algebraicType: AlgebraicType.Array(
          OneOffTable.getTypeScriptAlgebraicType()
        )
      },
      {
        name: "totalHostExecutionDuration",
        algebraicType: AlgebraicType.createTimeDurationType()
      }
    );
    return _cached_OneOffQueryResponse_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      OneOffQueryResponse.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      OneOffQueryResponse.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_SubscribeRows_type_value = null;
var SubscribeRows = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_SubscribeRows_type_value)
      return _cached_SubscribeRows_type_value;
    _cached_SubscribeRows_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_SubscribeRows_type_value.value.elements.push(
      { name: "tableId", algebraicType: AlgebraicType.U32 },
      { name: "tableName", algebraicType: AlgebraicType.String },
      {
        name: "tableRows",
        algebraicType: TableUpdate.getTypeScriptAlgebraicType()
      }
    );
    return _cached_SubscribeRows_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      SubscribeRows.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      SubscribeRows.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_SubscribeApplied_type_value = null;
var SubscribeApplied = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_SubscribeApplied_type_value)
      return _cached_SubscribeApplied_type_value;
    _cached_SubscribeApplied_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_SubscribeApplied_type_value.value.elements.push(
      { name: "requestId", algebraicType: AlgebraicType.U32 },
      {
        name: "totalHostExecutionDurationMicros",
        algebraicType: AlgebraicType.U64
      },
      { name: "queryId", algebraicType: QueryId.getTypeScriptAlgebraicType() },
      {
        name: "rows",
        algebraicType: SubscribeRows.getTypeScriptAlgebraicType()
      }
    );
    return _cached_SubscribeApplied_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      SubscribeApplied.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      SubscribeApplied.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_UnsubscribeApplied_type_value = null;
var UnsubscribeApplied = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_UnsubscribeApplied_type_value)
      return _cached_UnsubscribeApplied_type_value;
    _cached_UnsubscribeApplied_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_UnsubscribeApplied_type_value.value.elements.push(
      { name: "requestId", algebraicType: AlgebraicType.U32 },
      {
        name: "totalHostExecutionDurationMicros",
        algebraicType: AlgebraicType.U64
      },
      { name: "queryId", algebraicType: QueryId.getTypeScriptAlgebraicType() },
      {
        name: "rows",
        algebraicType: SubscribeRows.getTypeScriptAlgebraicType()
      }
    );
    return _cached_UnsubscribeApplied_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      UnsubscribeApplied.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      UnsubscribeApplied.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_SubscriptionError_type_value = null;
var SubscriptionError = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_SubscriptionError_type_value)
      return _cached_SubscriptionError_type_value;
    _cached_SubscriptionError_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_SubscriptionError_type_value.value.elements.push(
      {
        name: "totalHostExecutionDurationMicros",
        algebraicType: AlgebraicType.U64
      },
      {
        name: "requestId",
        algebraicType: AlgebraicType.createOptionType(
          AlgebraicType.U32
        )
      },
      {
        name: "queryId",
        algebraicType: AlgebraicType.createOptionType(
          AlgebraicType.U32
        )
      },
      {
        name: "tableId",
        algebraicType: AlgebraicType.createOptionType(
          AlgebraicType.U32
        )
      },
      { name: "error", algebraicType: AlgebraicType.String }
    );
    return _cached_SubscriptionError_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      SubscriptionError.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      SubscriptionError.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_SubscribeMultiApplied_type_value = null;
var SubscribeMultiApplied = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_SubscribeMultiApplied_type_value)
      return _cached_SubscribeMultiApplied_type_value;
    _cached_SubscribeMultiApplied_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_SubscribeMultiApplied_type_value.value.elements.push(
      { name: "requestId", algebraicType: AlgebraicType.U32 },
      {
        name: "totalHostExecutionDurationMicros",
        algebraicType: AlgebraicType.U64
      },
      { name: "queryId", algebraicType: QueryId.getTypeScriptAlgebraicType() },
      {
        name: "update",
        algebraicType: DatabaseUpdate.getTypeScriptAlgebraicType()
      }
    );
    return _cached_SubscribeMultiApplied_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      SubscribeMultiApplied.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      SubscribeMultiApplied.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_UnsubscribeMultiApplied_type_value = null;
var UnsubscribeMultiApplied = {
  /**
   * A function which returns this type represented as an AlgebraicType.
   * This function is derived from the AlgebraicType used to generate this type.
   */
  getTypeScriptAlgebraicType() {
    if (_cached_UnsubscribeMultiApplied_type_value)
      return _cached_UnsubscribeMultiApplied_type_value;
    _cached_UnsubscribeMultiApplied_type_value = AlgebraicType.Product({
      elements: []
    });
    _cached_UnsubscribeMultiApplied_type_value.value.elements.push(
      { name: "requestId", algebraicType: AlgebraicType.U32 },
      {
        name: "totalHostExecutionDurationMicros",
        algebraicType: AlgebraicType.U64
      },
      { name: "queryId", algebraicType: QueryId.getTypeScriptAlgebraicType() },
      {
        name: "update",
        algebraicType: DatabaseUpdate.getTypeScriptAlgebraicType()
      }
    );
    return _cached_UnsubscribeMultiApplied_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      UnsubscribeMultiApplied.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      UnsubscribeMultiApplied.getTypeScriptAlgebraicType()
    );
  }
};
var _cached_ServerMessage_type_value = null;
var ServerMessage = {
  // Helper functions for constructing each variant of the tagged union.
  // ```
  // const foo = Foo.A(42);
  // assert!(foo.tag === "A");
  // assert!(foo.value === 42);
  // ```
  InitialSubscription: (value) => ({
    tag: "InitialSubscription",
    value
  }),
  TransactionUpdate: (value) => ({
    tag: "TransactionUpdate",
    value
  }),
  TransactionUpdateLight: (value) => ({
    tag: "TransactionUpdateLight",
    value
  }),
  IdentityToken: (value) => ({ tag: "IdentityToken", value }),
  OneOffQueryResponse: (value) => ({
    tag: "OneOffQueryResponse",
    value
  }),
  SubscribeApplied: (value) => ({
    tag: "SubscribeApplied",
    value
  }),
  UnsubscribeApplied: (value) => ({
    tag: "UnsubscribeApplied",
    value
  }),
  SubscriptionError: (value) => ({
    tag: "SubscriptionError",
    value
  }),
  SubscribeMultiApplied: (value) => ({
    tag: "SubscribeMultiApplied",
    value
  }),
  UnsubscribeMultiApplied: (value) => ({
    tag: "UnsubscribeMultiApplied",
    value
  }),
  getTypeScriptAlgebraicType() {
    if (_cached_ServerMessage_type_value)
      return _cached_ServerMessage_type_value;
    _cached_ServerMessage_type_value = AlgebraicType.Sum({
      variants: []
    });
    _cached_ServerMessage_type_value.value.variants.push(
      {
        name: "InitialSubscription",
        algebraicType: InitialSubscription.getTypeScriptAlgebraicType()
      },
      {
        name: "TransactionUpdate",
        algebraicType: TransactionUpdate.getTypeScriptAlgebraicType()
      },
      {
        name: "TransactionUpdateLight",
        algebraicType: TransactionUpdateLight.getTypeScriptAlgebraicType()
      },
      {
        name: "IdentityToken",
        algebraicType: IdentityToken.getTypeScriptAlgebraicType()
      },
      {
        name: "OneOffQueryResponse",
        algebraicType: OneOffQueryResponse.getTypeScriptAlgebraicType()
      },
      {
        name: "SubscribeApplied",
        algebraicType: SubscribeApplied.getTypeScriptAlgebraicType()
      },
      {
        name: "UnsubscribeApplied",
        algebraicType: UnsubscribeApplied.getTypeScriptAlgebraicType()
      },
      {
        name: "SubscriptionError",
        algebraicType: SubscriptionError.getTypeScriptAlgebraicType()
      },
      {
        name: "SubscribeMultiApplied",
        algebraicType: SubscribeMultiApplied.getTypeScriptAlgebraicType()
      },
      {
        name: "UnsubscribeMultiApplied",
        algebraicType: UnsubscribeMultiApplied.getTypeScriptAlgebraicType()
      }
    );
    return _cached_ServerMessage_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(
      writer,
      ServerMessage.getTypeScriptAlgebraicType(),
      value
    );
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(
      reader,
      ServerMessage.getTypeScriptAlgebraicType()
    );
  }
};
var EventEmitter = class {
  #events = /* @__PURE__ */ new Map();
  on(event, callback) {
    let callbacks = this.#events.get(event);
    if (!callbacks) {
      callbacks = /* @__PURE__ */ new Set();
      this.#events.set(event, callbacks);
    }
    callbacks.add(callback);
  }
  off(event, callback) {
    const callbacks = this.#events.get(event);
    if (!callbacks) {
      return;
    }
    callbacks.delete(callback);
  }
  emit(event, ...args) {
    const callbacks = this.#events.get(event);
    if (!callbacks) {
      return;
    }
    for (const callback of callbacks) {
      callback(...args);
    }
  }
};
var LogLevelIdentifierIcon = {
  component: "\u{1F4E6}",
  info: "\u2139\uFE0F",
  warn: "\u26A0\uFE0F",
  error: "\u274C",
  debug: "\u{1F41B}"
};
var LogStyle = {
  component: "color: #fff; background-color: #8D6FDD; padding: 2px 5px; border-radius: 3px;",
  info: "color: #fff; background-color: #007bff; padding: 2px 5px; border-radius: 3px;",
  warn: "color: #fff; background-color: #ffc107; padding: 2px 5px; border-radius: 3px;",
  error: "color: #fff; background-color: #dc3545; padding: 2px 5px; border-radius: 3px;",
  debug: "color: #fff; background-color: #28a745; padding: 2px 5px; border-radius: 3px;"
};
var LogTextStyle = {
  component: "color: #8D6FDD;",
  info: "color: #007bff;",
  warn: "color: #ffc107;",
  error: "color: #dc3545;",
  debug: "color: #28a745;"
};
var stdbLogger = (level, message) => {
  console.log(
    `%c${LogLevelIdentifierIcon[level]} ${level.toUpperCase()}%c ${message}`,
    LogStyle[level],
    LogTextStyle[level]
  );
};
var TableCache = class {
  rows;
  tableTypeInfo;
  emitter;
  /**
   * @param name the table name
   * @param primaryKeyCol column index designated as `#[primarykey]`
   * @param primaryKey column name designated as `#[primarykey]`
   * @param entityClass the entityClass
   */
  constructor(tableTypeInfo) {
    this.tableTypeInfo = tableTypeInfo;
    this.rows = /* @__PURE__ */ new Map();
    this.emitter = new EventEmitter();
  }
  /**
   * @returns number of rows in the table
   */
  count() {
    return this.rows.size;
  }
  /**
   * @returns The values of the rows in the table
   */
  iter() {
    return Array.from(this.rows.values()).map(([row]) => row);
  }
  applyOperations = (operations, ctx) => {
    const pendingCallbacks = [];
    if (this.tableTypeInfo.primaryKeyInfo !== void 0) {
      const insertMap = /* @__PURE__ */ new Map();
      const deleteMap = /* @__PURE__ */ new Map();
      for (const op of operations) {
        if (op.type === "insert") {
          const [_, prevCount] = insertMap.get(op.rowId) || [op, 0];
          insertMap.set(op.rowId, [op, prevCount + 1]);
        } else {
          const [_, prevCount] = deleteMap.get(op.rowId) || [op, 0];
          deleteMap.set(op.rowId, [op, prevCount + 1]);
        }
      }
      for (const [primaryKey, [insertOp, refCount]] of insertMap) {
        const deleteEntry = deleteMap.get(primaryKey);
        if (deleteEntry) {
          const [_, deleteCount] = deleteEntry;
          const refCountDelta = refCount - deleteCount;
          const maybeCb = this.update(
            ctx,
            primaryKey,
            insertOp.row,
            refCountDelta
          );
          if (maybeCb) {
            pendingCallbacks.push(maybeCb);
          }
          deleteMap.delete(primaryKey);
        } else {
          const maybeCb = this.insert(ctx, insertOp, refCount);
          if (maybeCb) {
            pendingCallbacks.push(maybeCb);
          }
        }
      }
      for (const [deleteOp, refCount] of deleteMap.values()) {
        const maybeCb = this.delete(ctx, deleteOp, refCount);
        if (maybeCb) {
          pendingCallbacks.push(maybeCb);
        }
      }
    } else {
      for (const op of operations) {
        if (op.type === "insert") {
          const maybeCb = this.insert(ctx, op);
          if (maybeCb) {
            pendingCallbacks.push(maybeCb);
          }
        } else {
          const maybeCb = this.delete(ctx, op);
          if (maybeCb) {
            pendingCallbacks.push(maybeCb);
          }
        }
      }
    }
    return pendingCallbacks;
  };
  update = (ctx, rowId, newRow, refCountDelta = 0) => {
    const existingEntry = this.rows.get(rowId);
    if (!existingEntry) {
      stdbLogger(
        "error",
        `Updating a row that was not present in the cache. Table: ${this.tableTypeInfo.tableName}, RowId: ${rowId}`
      );
      return void 0;
    }
    const [oldRow, previousCount] = existingEntry;
    const refCount = Math.max(1, previousCount + refCountDelta);
    if (previousCount + refCountDelta <= 0) {
      stdbLogger(
        "error",
        `Negative reference count for in table ${this.tableTypeInfo.tableName} row ${rowId} (${previousCount} + ${refCountDelta})`
      );
      return void 0;
    }
    this.rows.set(rowId, [newRow, refCount]);
    if (previousCount === 0) {
      stdbLogger(
        "error",
        `Updating a row id in table ${this.tableTypeInfo.tableName} which was not present in the cache (rowId: ${rowId})`
      );
      return {
        type: "insert",
        table: this.tableTypeInfo.tableName,
        cb: () => {
          this.emitter.emit("insert", ctx, newRow);
        }
      };
    }
    return {
      type: "update",
      table: this.tableTypeInfo.tableName,
      cb: () => {
        this.emitter.emit("update", ctx, oldRow, newRow);
      }
    };
  };
  insert = (ctx, operation, count = 1) => {
    const [_, previousCount] = this.rows.get(operation.rowId) || [
      operation.row,
      0
    ];
    this.rows.set(operation.rowId, [operation.row, previousCount + count]);
    if (previousCount === 0) {
      return {
        type: "insert",
        table: this.tableTypeInfo.tableName,
        cb: () => {
          this.emitter.emit("insert", ctx, operation.row);
        }
      };
    }
    return void 0;
  };
  delete = (ctx, operation, count = 1) => {
    const [_, previousCount] = this.rows.get(operation.rowId) || [
      operation.row,
      0
    ];
    if (previousCount === 0) {
      stdbLogger("warn", "Deleting a row that was not present in the cache");
      return void 0;
    }
    if (previousCount <= count) {
      this.rows.delete(operation.rowId);
      return {
        type: "delete",
        table: this.tableTypeInfo.tableName,
        cb: () => {
          this.emitter.emit("delete", ctx, operation.row);
        }
      };
    }
    this.rows.set(operation.rowId, [operation.row, previousCount - count]);
    return void 0;
  };
  /**
   * Register a callback for when a row is newly inserted into the database.
   *
   * ```ts
   * User.onInsert((user, reducerEvent) => {
   *   if (reducerEvent) {
   *      console.log("New user on reducer", reducerEvent, user);
   *   } else {
   *      console.log("New user received during subscription update on insert", user);
   *  }
   * });
   * ```
   *
   * @param cb Callback to be called when a new row is inserted
   */
  onInsert = (cb) => {
    this.emitter.on("insert", cb);
  };
  /**
   * Register a callback for when a row is deleted from the database.
   *
   * ```ts
   * User.onDelete((user, reducerEvent) => {
   *   if (reducerEvent) {
   *      console.log("Deleted user on reducer", reducerEvent, user);
   *   } else {
   *      console.log("Deleted user received during subscription update on update", user);
   *  }
   * });
   * ```
   *
   * @param cb Callback to be called when a new row is inserted
   */
  onDelete = (cb) => {
    this.emitter.on("delete", cb);
  };
  /**
   * Register a callback for when a row is updated into the database.
   *
   * ```ts
   * User.onInsert((user, reducerEvent) => {
   *   if (reducerEvent) {
   *      console.log("Updated user on reducer", reducerEvent, user);
   *   } else {
   *      console.log("Updated user received during subscription update on delete", user);
   *  }
   * });
   * ```
   *
   * @param cb Callback to be called when a new row is inserted
   */
  onUpdate = (cb) => {
    this.emitter.on("update", cb);
  };
  /**
   * Remove a callback for when a row is newly inserted into the database.
   *
   * @param cb Callback to be removed
   */
  removeOnInsert = (cb) => {
    this.emitter.off("insert", cb);
  };
  /**
   * Remove a callback for when a row is deleted from the database.
   *
   * @param cb Callback to be removed
   */
  removeOnDelete = (cb) => {
    this.emitter.off("delete", cb);
  };
  /**
   * Remove a callback for when a row is updated into the database.
   *
   * @param cb Callback to be removed
   */
  removeOnUpdate = (cb) => {
    this.emitter.off("update", cb);
  };
};
var ClientCache = class {
  /**
   * The tables in the database.
   */
  tables;
  constructor() {
    this.tables = /* @__PURE__ */ new Map();
  }
  /**
   * Returns the table with the given name.
   * @param name The name of the table.
   * @returns The table
   */
  getTable(name) {
    const table = this.tables.get(name);
    if (!table) {
      console.error(
        "The table has not been registered for this client. Please register the table before using it. If you have registered global tables using the SpacetimeDBClient.registerTables() or `registerTable()` method, please make sure that is executed first!"
      );
      throw new Error(`Table ${name} does not exist`);
    }
    return table;
  }
  getOrCreateTable(tableTypeInfo) {
    let table;
    if (!this.tables.has(tableTypeInfo.tableName)) {
      table = new TableCache(tableTypeInfo);
      this.tables.set(tableTypeInfo.tableName, table);
    } else {
      table = this.tables.get(tableTypeInfo.tableName);
    }
    return table;
  }
};
function comparePreReleases(a, b) {
  const len = Math.min(a.length, b.length);
  for (let i = 0; i < len; i++) {
    const aPart = a[i];
    const bPart = b[i];
    if (aPart === bPart) continue;
    if (typeof aPart === "number" && typeof bPart === "number") {
      return aPart - bPart;
    }
    if (typeof aPart === "string" && typeof bPart === "string") {
      return aPart.localeCompare(bPart);
    }
    return typeof aPart === "string" ? 1 : -1;
  }
  return a.length - b.length;
}
var SemanticVersion = class _SemanticVersion {
  major;
  minor;
  patch;
  preRelease;
  buildInfo;
  constructor(major, minor, patch, preRelease = null, buildInfo = null) {
    this.major = major;
    this.minor = minor;
    this.patch = patch;
    this.preRelease = preRelease;
    this.buildInfo = buildInfo;
  }
  toString() {
    let versionString = `${this.major}.${this.minor}.${this.patch}`;
    if (this.preRelease) {
      versionString += `-${this.preRelease.join(".")}`;
    }
    if (this.buildInfo) {
      versionString += `+${this.buildInfo}`;
    }
    return versionString;
  }
  compare(other) {
    if (this.major !== other.major) {
      return this.major - other.major;
    }
    if (this.minor !== other.minor) {
      return this.minor - other.minor;
    }
    if (this.patch !== other.patch) {
      return this.patch - other.patch;
    }
    if (this.preRelease && other.preRelease) {
      return comparePreReleases(this.preRelease, other.preRelease);
    }
    if (this.preRelease) {
      return -1;
    }
    if (other.preRelease) {
      return -1;
    }
    return 0;
  }
  clone() {
    return new _SemanticVersion(
      this.major,
      this.minor,
      this.patch,
      this.preRelease ? [...this.preRelease] : null,
      this.buildInfo
    );
  }
  static parseVersionString(version) {
    const regex = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([\da-zA-Z-]+(?:\.[\da-zA-Z-]+)*))?(?:\+([\da-zA-Z-]+(?:\.[\da-zA-Z-]+)*))?$/;
    const match = version.match(regex);
    if (!match) {
      throw new Error(`Invalid version string: ${version}`);
    }
    const major = parseInt(match[1], 10);
    const minor = parseInt(match[2], 10);
    const patch = parseInt(match[3], 10);
    const preRelease = match[4] ? match[4].split(".").map((id) => isNaN(Number(id)) ? id : Number(id)) : null;
    const buildInfo = match[5] || null;
    return new _SemanticVersion(major, minor, patch, preRelease, buildInfo);
  }
};
var _MINIMUM_CLI_VERSION = new SemanticVersion(
  1,
  4,
  0
);
function ensureMinimumVersionOrThrow(versionString) {
  if (versionString === void 0) {
    throw new Error(versionErrorMessage(versionString));
  }
  const version = SemanticVersion.parseVersionString(versionString);
  if (version.compare(_MINIMUM_CLI_VERSION) < 0) {
    throw new Error(versionErrorMessage(versionString));
  }
}
function versionErrorMessage(incompatibleVersion) {
  return `Module code was generated with an incompatible version of the spacetimedb cli (${incompatibleVersion}). Update the cli version to at least ${_MINIMUM_CLI_VERSION.toString()} and regenerate the bindings. You can upgrade to the latest cli version by running: spacetime version upgrade`;
}
async function decompress(buffer, type, chunkSize = 128 * 1024) {
  let offset = 0;
  const readableStream = new ReadableStream({
    pull(controller) {
      if (offset < buffer.length) {
        const chunk = buffer.subarray(
          offset,
          Math.min(offset + chunkSize, buffer.length)
        );
        controller.enqueue(chunk);
        offset += chunkSize;
      } else {
        controller.close();
      }
    }
  });
  const decompressionStream = new DecompressionStream(type);
  const decompressedStream = readableStream.pipeThrough(decompressionStream);
  const reader = decompressedStream.getReader();
  const chunks = [];
  let totalLength = 0;
  let result;
  while (!(result = await reader.read()).done) {
    chunks.push(result.value);
    totalLength += result.value.length;
  }
  const decompressedArray = new Uint8Array(totalLength);
  let chunkOffset = 0;
  for (const chunk of chunks) {
    decompressedArray.set(chunk, chunkOffset);
    chunkOffset += chunk.length;
  }
  return decompressedArray;
}
async function resolveWS() {
  if (typeof globalThis.WebSocket !== "undefined") {
    return globalThis.WebSocket;
  }
  const dynamicImport = new Function("m", "return import(m)");
  try {
    const { WebSocket: UndiciWS } = await dynamicImport("undici");
    return UndiciWS;
  } catch (err) {
    console.warn(
      "[spacetimedb-sdk] No global WebSocket found. On Node 18\u201321, please install `undici` (npm install undici) to enable WebSocket support."
    );
    throw err;
  }
}
var WebsocketDecompressAdapter = class _WebsocketDecompressAdapter {
  onclose;
  onopen;
  onmessage;
  onerror;
  #ws;
  async #handleOnMessage(msg) {
    const buffer = new Uint8Array(msg.data);
    let decompressed;
    if (buffer[0] === 0) {
      decompressed = buffer.slice(1);
    } else if (buffer[0] === 1) {
      throw new Error(
        "Brotli Compression not supported. Please use gzip or none compression in withCompression method on DbConnection."
      );
    } else if (buffer[0] === 2) {
      decompressed = await decompress(buffer.slice(1), "gzip");
    } else {
      throw new Error(
        "Unexpected Compression Algorithm. Please use `gzip` or `none`"
      );
    }
    this.onmessage?.({ data: decompressed });
  }
  #handleOnOpen(msg) {
    this.onopen?.(msg);
  }
  #handleOnError(msg) {
    this.onerror?.(msg);
  }
  #handleOnClose(msg) {
    this.onclose?.(msg);
  }
  send(msg) {
    this.#ws.send(msg);
  }
  close() {
    this.#ws.close();
  }
  constructor(ws) {
    this.onmessage = void 0;
    this.onopen = void 0;
    this.onmessage = void 0;
    this.onerror = void 0;
    ws.onmessage = this.#handleOnMessage.bind(this);
    ws.onerror = this.#handleOnError.bind(this);
    ws.onclose = this.#handleOnClose.bind(this);
    ws.onopen = this.#handleOnOpen.bind(this);
    ws.binaryType = "arraybuffer";
    this.#ws = ws;
  }
  static async createWebSocketFn({
    url,
    nameOrAddress,
    wsProtocol,
    authToken,
    compression,
    lightMode,
    confirmedReads
  }) {
    const headers = new Headers();
    const WS = await resolveWS();
    let temporaryAuthToken = void 0;
    if (authToken) {
      headers.set("Authorization", `Bearer ${authToken}`);
      const tokenUrl = new URL("v1/identity/websocket-token", url);
      tokenUrl.protocol = url.protocol === "wss:" ? "https:" : "http:";
      const response = await fetch(tokenUrl, { method: "POST", headers });
      if (response.ok) {
        const { token } = await response.json();
        temporaryAuthToken = token;
      } else {
        return Promise.reject(
          new Error(`Failed to verify token: ${response.statusText}`)
        );
      }
    }
    const databaseUrl = new URL(`v1/database/${nameOrAddress}/subscribe`, url);
    if (temporaryAuthToken) {
      databaseUrl.searchParams.set("token", temporaryAuthToken);
    }
    databaseUrl.searchParams.set(
      "compression",
      compression === "gzip" ? "Gzip" : "None"
    );
    if (lightMode) {
      databaseUrl.searchParams.set("light", "true");
    }
    if (confirmedReads !== void 0) {
      databaseUrl.searchParams.set("confirmed", confirmedReads.toString());
    }
    const ws = new WS(databaseUrl.toString(), wsProtocol);
    return new _WebsocketDecompressAdapter(ws);
  }
};
var DbConnectionBuilder = class {
  /**
   * Creates a new `DbConnectionBuilder` database client and set the initial parameters.
   *
   * Users are not expected to call this constructor directly. Instead, use the static method `DbConnection.builder()`.
   *
   * @param remoteModule The remote module to use to connect to the SpacetimeDB server.
   * @param dbConnectionConstructor The constructor to use to create a new `DbConnection`.
   */
  constructor(remoteModule, dbConnectionConstructor) {
    this.remoteModule = remoteModule;
    this.dbConnectionConstructor = dbConnectionConstructor;
    this.#createWSFn = WebsocketDecompressAdapter.createWebSocketFn;
  }
  #uri;
  #nameOrAddress;
  #identity;
  #token;
  #emitter = new EventEmitter();
  #compression = "gzip";
  #lightMode = false;
  #confirmedReads;
  #createWSFn;
  /**
   * Set the URI of the SpacetimeDB server to connect to.
   *
   * @param uri The URI of the SpacetimeDB server to connect to.
   *
   **/
  withUri(uri) {
    this.#uri = new URL(uri);
    return this;
  }
  /**
   * Set the name or Identity of the database module to connect to.
   *
   * @param nameOrAddress
   *
   * @returns The `DbConnectionBuilder` instance.
   */
  withModuleName(nameOrAddress) {
    this.#nameOrAddress = nameOrAddress;
    return this;
  }
  /**
   * Set the identity of the client to connect to the database.
   *
   * @param token The credentials to use to authenticate with SpacetimeDB. This
   * is optional. You can store the token returned by the `onConnect` callback
   * to use in future connections.
   *
   * @returns The `DbConnectionBuilder` instance.
   */
  withToken(token) {
    this.#token = token;
    return this;
  }
  withWSFn(createWSFn) {
    this.#createWSFn = createWSFn;
    return this;
  }
  /**
   * Set the compression algorithm to use for the connection.
   *
   * @param compression The compression algorithm to use for the connection.
   */
  withCompression(compression) {
    this.#compression = compression;
    return this;
  }
  /**
   * Sets the connection to operate in light mode.
   *
   * Light mode is a mode that reduces the amount of data sent over the network.
   *
   * @param lightMode The light mode for the connection.
   */
  withLightMode(lightMode) {
    this.#lightMode = lightMode;
    return this;
  }
  /**
   * Sets the connection to use confirmed reads.
   *
   * When enabled, the server will send query results only after they are
   * confirmed to be durable.
   *
   * What durable means depends on the server configuration: a single node
   * server may consider a transaction durable once it is `fsync`'ed to disk,
   * whereas a cluster may require that some number of replicas have
   * acknowledge that they have stored the transactions.
   *
   * Note that enabling confirmed reads will increase the latency between a
   * reducer call and the corresponding subscription update arriving at the
   * client.
   *
   * If this method is not called, not preference is sent to the server, and
   * the server will choose the default.
   *
   * @param confirmedReads `true` to enable confirmed reads, `false` to disable.
   */
  withConfirmedReads(confirmedReads) {
    this.#confirmedReads = confirmedReads;
    return this;
  }
  /**
   * Register a callback to be invoked upon authentication with the database.
   *
   * @param identity A unique identifier for a client connected to a database.
   * @param token The credentials to use to authenticate with SpacetimeDB.
   *
   * @returns The `DbConnectionBuilder` instance.
   *
   * The callback will be invoked with the `Identity` and private authentication `token` provided by the database to identify this connection.
   *
   * If credentials were supplied to connect, those passed to the callback will be equivalent to the ones used to connect.
   *
   * If the initial connection was anonymous, a new set of credentials will be generated by the database to identify this user.
   *
   * The credentials passed to the callback can be saved and used to authenticate the same user in future connections.
   *
   * @example
   *
   * ```ts
   * DbConnection.builder().onConnect((ctx, identity, token) => {
   *  console.log("Connected to SpacetimeDB with identity:", identity.toHexString());
   * });
   * ```
   */
  onConnect(callback) {
    this.#emitter.on("connect", callback);
    return this;
  }
  /**
   * Register a callback to be invoked upon an error.
   *
   * @example
   *
   * ```ts
   * DbConnection.builder().onConnectError((ctx, error) => {
   *   console.log("Error connecting to SpacetimeDB:", error);
   * });
   * ```
   */
  onConnectError(callback) {
    this.#emitter.on("connectError", callback);
    return this;
  }
  /**
   * Registers a callback to run when a {@link DbConnection} whose connection initially succeeded
   * is disconnected, either after a {@link DbConnection.disconnect} call or due to an error.
   *
   * If the connection ended because of an error, the error is passed to the callback.
   *
   * The `callback` will be installed on the `DbConnection` created by `build`
   * before initiating the connection, ensuring there's no opportunity for the disconnect to happen
   * before the callback is installed.
   *
   * Note that this does not trigger if `build` fails
   * or in cases where {@link DbConnectionBuilder.onConnectError} would trigger.
   * This callback only triggers if the connection closes after `build` returns successfully
   * and {@link DbConnectionBuilder.onConnect} is invoked, i.e., after the `IdentityToken` is received.
   *
   * To simplify SDK implementation, at most one such callback can be registered.
   * Calling `onDisconnect` on the same `DbConnectionBuilder` multiple times throws an error.
   *
   * Unlike callbacks registered via {@link DbConnection},
   * no mechanism is provided to unregister the provided callback.
   * This is a concession to ergonomics; there's no clean place to return a `CallbackId` from this method
   * or from `build`.
   *
   * @param {function(error?: Error): void} callback - The callback to invoke upon disconnection.
   * @throws {Error} Throws an error if called multiple times on the same `DbConnectionBuilder`.
   */
  onDisconnect(callback) {
    this.#emitter.on("disconnect", callback);
    return this;
  }
  /**
   * Builds a new `DbConnection` with the parameters set on this `DbConnectionBuilder` and attempts to connect to the SpacetimeDB server.
   *
   * @returns A new `DbConnection` with the parameters set on this `DbConnectionBuilder`.
   *
   * @example
   *
   * ```ts
   * const host = "http://localhost:3000";
   * const name_or_address = "database_name"
   * const auth_token = undefined;
   * DbConnection.builder().withUri(host).withModuleName(name_or_address).withToken(auth_token).build();
   * ```
   */
  build() {
    if (!this.#uri) {
      throw new Error("URI is required to connect to SpacetimeDB");
    }
    if (!this.#nameOrAddress) {
      throw new Error(
        "Database name or address is required to connect to SpacetimeDB"
      );
    }
    ensureMinimumVersionOrThrow(this.remoteModule.versionInfo?.cliVersion);
    return this.dbConnectionConstructor(
      new DbConnectionImpl({
        uri: this.#uri,
        nameOrAddress: this.#nameOrAddress,
        identity: this.#identity,
        token: this.#token,
        emitter: this.#emitter,
        compression: this.#compression,
        lightMode: this.#lightMode,
        confirmedReads: this.#confirmedReads,
        createWSFn: this.#createWSFn,
        remoteModule: this.remoteModule
      })
    );
  }
};
var SubscriptionBuilderImpl = class {
  constructor(db) {
    this.db = db;
  }
  #onApplied = void 0;
  #onError = void 0;
  /**
   * Registers `callback` to run when this query is successfully added to our subscribed set,
   * I.e. when its `SubscriptionApplied` message is received.
   *
   * The database state exposed via the `&EventContext` argument
   * includes all the rows added to the client cache as a result of the new subscription.
   *
   * The event in the `&EventContext` argument is `Event::SubscribeApplied`.
   *
   * Multiple `on_applied` callbacks for the same query may coexist.
   * No mechanism for un-registering `on_applied` callbacks is exposed.
   *
   * @param cb - Callback to run when the subscription is applied.
   * @returns The current `SubscriptionBuilder` instance.
   */
  onApplied(cb) {
    this.#onApplied = cb;
    return this;
  }
  /**
   * Registers `callback` to run when this query either:
   * - Fails to be added to our subscribed set.
   * - Is unexpectedly removed from our subscribed set.
   *
   * If the subscription had previously started and has been unexpectedly removed,
   * the database state exposed via the `&EventContext` argument contains no rows
   * from any subscriptions removed within the same error event.
   * As proposed, it must therefore contain no rows.
   *
   * The event in the `&EventContext` argument is `Event::SubscribeError`,
   * containing a dynamic error object with a human-readable description of the error
   * for diagnostic purposes.
   *
   * Multiple `on_error` callbacks for the same query may coexist.
   * No mechanism for un-registering `on_error` callbacks is exposed.
   *
   * @param cb - Callback to run when there is an error in subscription.
   * @returns The current `SubscriptionBuilder` instance.
   */
  onError(cb) {
    this.#onError = cb;
    return this;
  }
  /**
   * Subscribe to a single query. The results of the query will be merged into the client
   * cache and deduplicated on the client.
   *
   * @param query_sql A `SQL` query to subscribe to.
   *
   * @example
   *
   * ```ts
   * const subscription = connection.subscriptionBuilder().onApplied(() => {
   *   console.log("SDK client cache initialized.");
   * }).subscribe("SELECT * FROM User");
   *
   * subscription.unsubscribe();
   * ```
   */
  subscribe(query_sql) {
    const queries = Array.isArray(query_sql) ? query_sql : [query_sql];
    if (queries.length === 0) {
      throw new Error("Subscriptions must have at least one query");
    }
    return new SubscriptionHandleImpl(
      this.db,
      queries,
      this.#onApplied,
      this.#onError
    );
  }
  /**
   * Subscribes to all rows from all tables.
   *
   * This method is intended as a convenience
   * for applications where client-side memory use and network bandwidth are not concerns.
   * Applications where these resources are a constraint
   * should register more precise queries via `subscribe`
   * in order to replicate only the subset of data which the client needs to function.
   *
   * This method should not be combined with `subscribe` on the same `DbConnection`.
   * A connection may either `subscribe` to particular queries,
   * or `subscribeToAllTables`, but not both.
   * Attempting to call `subscribe`
   * on a `DbConnection` that has previously used `subscribeToAllTables`,
   * or vice versa, may misbehave in any number of ways,
   * including dropping subscriptions, corrupting the client cache, or throwing errors.
   */
  subscribeToAllTables() {
    this.subscribe("SELECT * FROM *");
  }
};
var SubscriptionManager = class {
  subscriptions = /* @__PURE__ */ new Map();
};
var SubscriptionHandleImpl = class {
  constructor(db, querySql, onApplied, onError) {
    this.db = db;
    this.#emitter.on(
      "applied",
      (ctx) => {
        this.#activeState = true;
        if (onApplied) {
          onApplied(ctx);
        }
      }
    );
    this.#emitter.on(
      "error",
      (ctx, error) => {
        this.#activeState = false;
        this.#endedState = true;
        if (onError) {
          onError(ctx, error);
        }
      }
    );
    this.#queryId = this.db.registerSubscription(this, this.#emitter, querySql);
  }
  #queryId;
  #unsubscribeCalled = false;
  #endedState = false;
  #activeState = false;
  #emitter = new EventEmitter();
  /**
   * Consumes self and issues an `Unsubscribe` message,
   * removing this query from the client's set of subscribed queries.
   * It is only valid to call this method if `is_active()` is `true`.
   */
  unsubscribe() {
    if (this.#unsubscribeCalled) {
      throw new Error("Unsubscribe has already been called");
    }
    this.#unsubscribeCalled = true;
    this.db.unregisterSubscription(this.#queryId);
    this.#emitter.on(
      "end",
      (_ctx) => {
        this.#endedState = true;
        this.#activeState = false;
      }
    );
  }
  /**
   * Unsubscribes and also registers a callback to run upon success.
   * I.e. when an `UnsubscribeApplied` message is received.
   *
   * If `Unsubscribe` returns an error,
   * or if the `on_error` callback(s) are invoked before this subscription would end normally,
   * the `on_end` callback is not invoked.
   *
   * @param onEnd - Callback to run upon successful unsubscribe.
   */
  unsubscribeThen(onEnd) {
    if (this.#endedState) {
      throw new Error("Subscription has already ended");
    }
    if (this.#unsubscribeCalled) {
      throw new Error("Unsubscribe has already been called");
    }
    this.#unsubscribeCalled = true;
    this.db.unregisterSubscription(this.#queryId);
    this.#emitter.on(
      "end",
      (ctx) => {
        this.#endedState = true;
        this.#activeState = false;
        onEnd(ctx);
      }
    );
  }
  /**
   * True if this `SubscriptionHandle` has ended,
   * either due to an error or a call to `unsubscribe`.
   *
   * This is initially false, and becomes true when either the `on_end` or `on_error` callback is invoked.
   * A subscription which has not yet been applied is not active, but is also not ended.
   */
  isEnded() {
    return this.#endedState;
  }
  /**
   * True if this `SubscriptionHandle` is active, meaning it has been successfully applied
   * and has not since ended, either due to an error or a complete `unsubscribe` request-response pair.
   *
   * This corresponds exactly to the interval bounded at the start by the `on_applied` callback
   * and at the end by either the `on_end` or `on_error` callback.
   */
  isActive() {
    return this.#activeState;
  }
};
function callReducerFlagsToNumber(flags) {
  switch (flags) {
    case "FullUpdate":
      return 0;
    case "NoSuccessNotify":
      return 1;
  }
}
var DbConnectionImpl = class {
  /**
   * Whether or not the connection is active.
   */
  isActive = false;
  /**
   * This connection's public identity.
   */
  identity = void 0;
  /**
   * This connection's private authentication token.
   */
  token = void 0;
  /**
   * The accessor field to access the tables in the database and associated
   * callback functions.
   */
  db;
  /**
   * The accessor field to access the reducers in the database and associated
   * callback functions.
   */
  reducers;
  /**
   * The accessor field to access functions related to setting flags on
   * reducers regarding how the server should handle the reducer call and
   * the events that it sends back to the client.
   */
  setReducerFlags;
  /**
   * The `ConnectionId` of the connection to to the database.
   */
  connectionId = ConnectionId.random();
  // These fields are meant to be strictly private.
  #queryId = 0;
  #emitter;
  #reducerEmitter = new EventEmitter();
  #onApplied;
  #remoteModule;
  #messageQueue = Promise.resolve();
  #subscriptionManager = new SubscriptionManager();
  // These fields are not part of the public API, but in a pinch you
  // could use JavaScript to access them by bypassing TypeScript's
  // private fields.
  // We use them in testing.
  clientCache;
  ws;
  wsPromise;
  constructor({
    uri,
    nameOrAddress,
    identity,
    token,
    emitter,
    remoteModule,
    createWSFn,
    compression,
    lightMode,
    confirmedReads
  }) {
    stdbLogger("info", "Connecting to SpacetimeDB WS...");
    const url = new URL(uri.toString());
    if (!/^wss?:/.test(uri.protocol)) {
      url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
    }
    this.identity = identity;
    this.token = token;
    this.#remoteModule = remoteModule;
    this.#emitter = emitter;
    const connectionId = this.connectionId.toHexString();
    url.searchParams.set("connection_id", connectionId);
    this.clientCache = new ClientCache();
    this.db = this.#remoteModule.dbViewConstructor(this);
    this.setReducerFlags = this.#remoteModule.setReducerFlagsConstructor();
    this.reducers = this.#remoteModule.reducersConstructor(
      this,
      this.setReducerFlags
    );
    this.wsPromise = createWSFn({
      url,
      nameOrAddress,
      wsProtocol: "v1.bsatn.spacetimedb",
      authToken: token,
      compression,
      lightMode,
      confirmedReads
    }).then((v) => {
      this.ws = v;
      this.ws.onclose = () => {
        this.#emitter.emit("disconnect", this);
      };
      this.ws.onerror = (e) => {
        this.#emitter.emit("connectError", this, e);
      };
      this.ws.onopen = this.#handleOnOpen.bind(this);
      this.ws.onmessage = this.#handleOnMessage.bind(this);
      return v;
    }).catch((e) => {
      stdbLogger("error", "Error connecting to SpacetimeDB WS");
      this.#emitter.emit("connectError", this, e);
      return void 0;
    });
  }
  #getNextQueryId = () => {
    const queryId = this.#queryId;
    this.#queryId += 1;
    return queryId;
  };
  // NOTE: This is very important!!! This is the actual function that
  // gets called when you call `connection.subscriptionBuilder()`.
  // The `subscriptionBuilder` function which is generated, just shadows
  // this function in the type system, but not the actual implementation!
  // Do not remove this function, or shoot yourself in the foot please.
  // It's not clear what would be a better way to do this at this exact
  // moment.
  subscriptionBuilder = () => {
    return new SubscriptionBuilderImpl(this);
  };
  registerSubscription(handle, handleEmitter, querySql) {
    const queryId = this.#getNextQueryId();
    this.#subscriptionManager.subscriptions.set(queryId, {
      handle,
      emitter: handleEmitter
    });
    this.#sendMessage(
      ClientMessage.SubscribeMulti({
        queryStrings: querySql,
        queryId: { id: queryId },
        // The TypeScript SDK doesn't currently track `request_id`s,
        // so always use 0.
        requestId: 0
      })
    );
    return queryId;
  }
  unregisterSubscription(queryId) {
    this.#sendMessage(
      ClientMessage.UnsubscribeMulti({
        queryId: { id: queryId },
        // The TypeScript SDK doesn't currently track `request_id`s,
        // so always use 0.
        requestId: 0
      })
    );
  }
  // This function is async because we decompress the message async
  async #processParsedMessage(message) {
    const parseRowList = (type, tableName, rowList) => {
      const buffer = rowList.rowsData;
      const reader = new BinaryReader(buffer);
      const rows = [];
      const rowType = this.#remoteModule.tables[tableName].rowType;
      let previousOffset = 0;
      const primaryKeyInfo = this.#remoteModule.tables[tableName].primaryKeyInfo;
      while (reader.remaining > 0) {
        const row = AlgebraicType.deserializeValue(reader, rowType);
        let rowId = void 0;
        if (primaryKeyInfo !== void 0) {
          rowId = AlgebraicType.intoMapKey(
            primaryKeyInfo.colType,
            row[primaryKeyInfo.colName]
          );
        } else {
          const rowBytes = buffer.subarray(previousOffset, reader.offset);
          const asBase64 = (0, import_base64_js.fromByteArray)(rowBytes);
          rowId = asBase64;
        }
        previousOffset = reader.offset;
        rows.push({
          type,
          rowId,
          row
        });
      }
      return rows;
    };
    const parseTableUpdate = async (rawTableUpdate) => {
      const tableName = rawTableUpdate.tableName;
      let operations = [];
      for (const update of rawTableUpdate.updates) {
        let decompressed;
        if (update.tag === "Gzip") {
          const decompressedBuffer = await decompress(update.value, "gzip");
          decompressed = QueryUpdate.deserialize(
            new BinaryReader(decompressedBuffer)
          );
        } else if (update.tag === "Brotli") {
          throw new Error(
            "Brotli compression not supported. Please use gzip or none compression in withCompression method on DbConnection."
          );
        } else {
          decompressed = update.value;
        }
        operations = operations.concat(
          parseRowList("insert", tableName, decompressed.inserts)
        );
        operations = operations.concat(
          parseRowList("delete", tableName, decompressed.deletes)
        );
      }
      return {
        tableName,
        operations
      };
    };
    const parseDatabaseUpdate = async (dbUpdate) => {
      const tableUpdates = [];
      for (const rawTableUpdate of dbUpdate.tables) {
        tableUpdates.push(await parseTableUpdate(rawTableUpdate));
      }
      return tableUpdates;
    };
    switch (message.tag) {
      case "InitialSubscription": {
        const dbUpdate = message.value.databaseUpdate;
        const tableUpdates = await parseDatabaseUpdate(dbUpdate);
        const subscriptionUpdate = {
          tag: "InitialSubscription",
          tableUpdates
        };
        return subscriptionUpdate;
      }
      case "TransactionUpdateLight": {
        const dbUpdate = message.value.update;
        const tableUpdates = await parseDatabaseUpdate(dbUpdate);
        const subscriptionUpdate = {
          tag: "TransactionUpdateLight",
          tableUpdates
        };
        return subscriptionUpdate;
      }
      case "TransactionUpdate": {
        const txUpdate = message.value;
        const identity = txUpdate.callerIdentity;
        const connectionId = ConnectionId.nullIfZero(
          txUpdate.callerConnectionId
        );
        const reducerName = txUpdate.reducerCall.reducerName;
        const args = txUpdate.reducerCall.args;
        const energyQuantaUsed = txUpdate.energyQuantaUsed;
        let tableUpdates = [];
        let errMessage = "";
        switch (txUpdate.status.tag) {
          case "Committed":
            tableUpdates = await parseDatabaseUpdate(txUpdate.status.value);
            break;
          case "Failed":
            tableUpdates = [];
            errMessage = txUpdate.status.value;
            break;
          case "OutOfEnergy":
            tableUpdates = [];
            break;
        }
        if (reducerName === "<none>") {
          const errorMessage = errMessage;
          console.error(`Received an error from the database: ${errorMessage}`);
          return;
        }
        let reducerInfo;
        if (reducerName !== "") {
          reducerInfo = {
            reducerName,
            args
          };
        }
        const transactionUpdate = {
          tag: "TransactionUpdate",
          tableUpdates,
          identity,
          connectionId,
          reducerInfo,
          status: txUpdate.status,
          energyConsumed: energyQuantaUsed.quanta,
          message: errMessage,
          timestamp: txUpdate.timestamp
        };
        return transactionUpdate;
      }
      case "IdentityToken": {
        const identityTokenMessage = {
          tag: "IdentityToken",
          identity: message.value.identity,
          token: message.value.token,
          connectionId: message.value.connectionId
        };
        return identityTokenMessage;
      }
      case "OneOffQueryResponse": {
        throw new Error(
          `TypeScript SDK never sends one-off queries, but got OneOffQueryResponse ${message}`
        );
      }
      case "SubscribeMultiApplied": {
        const parsedTableUpdates = await parseDatabaseUpdate(
          message.value.update
        );
        const subscribeAppliedMessage = {
          tag: "SubscribeApplied",
          queryId: message.value.queryId.id,
          tableUpdates: parsedTableUpdates
        };
        return subscribeAppliedMessage;
      }
      case "UnsubscribeMultiApplied": {
        const parsedTableUpdates = await parseDatabaseUpdate(
          message.value.update
        );
        const unsubscribeAppliedMessage = {
          tag: "UnsubscribeApplied",
          queryId: message.value.queryId.id,
          tableUpdates: parsedTableUpdates
        };
        return unsubscribeAppliedMessage;
      }
      case "SubscriptionError": {
        return {
          tag: "SubscriptionError",
          queryId: message.value.queryId,
          error: message.value.error
        };
      }
    }
  }
  #sendMessage(message) {
    this.wsPromise.then((wsResolved) => {
      if (wsResolved) {
        const writer = new BinaryWriter(1024);
        ClientMessage.serialize(writer, message);
        const encoded = writer.getBuffer();
        wsResolved.send(encoded);
      }
    });
  }
  /**
   * Handles WebSocket onOpen event.
   */
  #handleOnOpen() {
    this.isActive = true;
  }
  #applyTableUpdates(tableUpdates, eventContext) {
    const pendingCallbacks = [];
    for (const tableUpdate of tableUpdates) {
      const tableName = tableUpdate.tableName;
      const tableTypeInfo = this.#remoteModule.tables[tableName];
      const table = this.clientCache.getOrCreateTable(tableTypeInfo);
      const newCallbacks = table.applyOperations(
        tableUpdate.operations,
        eventContext
      );
      for (const callback of newCallbacks) {
        pendingCallbacks.push(callback);
      }
    }
    return pendingCallbacks;
  }
  async #processMessage(data) {
    const serverMessage = parseValue(ServerMessage, data);
    const message = await this.#processParsedMessage(serverMessage);
    if (!message) {
      return;
    }
    switch (message.tag) {
      case "InitialSubscription": {
        const event = { tag: "SubscribeApplied" };
        const eventContext = this.#remoteModule.eventContextConstructor(
          this,
          event
        );
        const { event: _, ...subscriptionEventContext } = eventContext;
        const callbacks = this.#applyTableUpdates(
          message.tableUpdates,
          eventContext
        );
        if (this.#emitter) {
          this.#onApplied?.(subscriptionEventContext);
        }
        for (const callback of callbacks) {
          callback.cb();
        }
        break;
      }
      case "TransactionUpdateLight": {
        const event = { tag: "UnknownTransaction" };
        const eventContext = this.#remoteModule.eventContextConstructor(
          this,
          event
        );
        const callbacks = this.#applyTableUpdates(
          message.tableUpdates,
          eventContext
        );
        for (const callback of callbacks) {
          callback.cb();
        }
        break;
      }
      case "TransactionUpdate": {
        let reducerInfo = message.reducerInfo;
        let unknownTransaction = false;
        let reducerArgs;
        let reducerTypeInfo;
        if (!reducerInfo) {
          unknownTransaction = true;
        } else {
          reducerTypeInfo = this.#remoteModule.reducers[reducerInfo.reducerName];
          try {
            const reader = new BinaryReader(reducerInfo.args);
            reducerArgs = AlgebraicType.deserializeValue(
              reader,
              reducerTypeInfo.argsType
            );
          } catch {
            console.debug("Failed to deserialize reducer arguments");
            unknownTransaction = true;
          }
        }
        if (unknownTransaction) {
          const event2 = { tag: "UnknownTransaction" };
          const eventContext2 = this.#remoteModule.eventContextConstructor(
            this,
            event2
          );
          const callbacks2 = this.#applyTableUpdates(
            message.tableUpdates,
            eventContext2
          );
          for (const callback of callbacks2) {
            callback.cb();
          }
          return;
        }
        reducerInfo = reducerInfo;
        reducerTypeInfo = reducerTypeInfo;
        const reducerEvent = {
          callerIdentity: message.identity,
          status: message.status,
          callerConnectionId: message.connectionId,
          timestamp: message.timestamp,
          energyConsumed: message.energyConsumed,
          reducer: {
            name: reducerInfo.reducerName,
            args: reducerArgs
          }
        };
        const event = {
          tag: "Reducer",
          value: reducerEvent
        };
        const eventContext = this.#remoteModule.eventContextConstructor(
          this,
          event
        );
        const reducerEventContext = {
          ...eventContext,
          event: reducerEvent
        };
        const callbacks = this.#applyTableUpdates(
          message.tableUpdates,
          eventContext
        );
        const argsArray = [];
        reducerTypeInfo.argsType.value.elements.forEach((element) => {
          argsArray.push(reducerArgs[element.name]);
        });
        this.#reducerEmitter.emit(
          reducerInfo.reducerName,
          reducerEventContext,
          ...argsArray
        );
        for (const callback of callbacks) {
          callback.cb();
        }
        break;
      }
      case "IdentityToken": {
        this.identity = message.identity;
        if (!this.token && message.token) {
          this.token = message.token;
        }
        this.connectionId = message.connectionId;
        this.#emitter.emit("connect", this, this.identity, this.token);
        break;
      }
      case "SubscribeApplied": {
        const subscription = this.#subscriptionManager.subscriptions.get(
          message.queryId
        );
        if (subscription === void 0) {
          stdbLogger(
            "error",
            `Received SubscribeApplied for unknown queryId ${message.queryId}.`
          );
          break;
        }
        const event = { tag: "SubscribeApplied" };
        const eventContext = this.#remoteModule.eventContextConstructor(
          this,
          event
        );
        const { event: _, ...subscriptionEventContext } = eventContext;
        const callbacks = this.#applyTableUpdates(
          message.tableUpdates,
          eventContext
        );
        subscription?.emitter.emit("applied", subscriptionEventContext);
        for (const callback of callbacks) {
          callback.cb();
        }
        break;
      }
      case "UnsubscribeApplied": {
        const subscription = this.#subscriptionManager.subscriptions.get(
          message.queryId
        );
        if (subscription === void 0) {
          stdbLogger(
            "error",
            `Received UnsubscribeApplied for unknown queryId ${message.queryId}.`
          );
          break;
        }
        const event = { tag: "UnsubscribeApplied" };
        const eventContext = this.#remoteModule.eventContextConstructor(
          this,
          event
        );
        const { event: _, ...subscriptionEventContext } = eventContext;
        const callbacks = this.#applyTableUpdates(
          message.tableUpdates,
          eventContext
        );
        subscription?.emitter.emit("end", subscriptionEventContext);
        this.#subscriptionManager.subscriptions.delete(message.queryId);
        for (const callback of callbacks) {
          callback.cb();
        }
        break;
      }
      case "SubscriptionError": {
        const error = Error(message.error);
        const event = { tag: "Error", value: error };
        const eventContext = this.#remoteModule.eventContextConstructor(
          this,
          event
        );
        const errorContext = {
          ...eventContext,
          event: error
        };
        if (message.queryId !== void 0) {
          this.#subscriptionManager.subscriptions.get(message.queryId)?.emitter.emit("error", errorContext, error);
          this.#subscriptionManager.subscriptions.delete(message.queryId);
        } else {
          console.error("Received an error message without a queryId: ", error);
          this.#subscriptionManager.subscriptions.forEach(({ emitter }) => {
            emitter.emit("error", errorContext, error);
          });
        }
      }
    }
  }
  /**
   * Handles WebSocket onMessage event.
   * @param wsMessage MessageEvent object.
   */
  #handleOnMessage(wsMessage) {
    this.#messageQueue = this.#messageQueue.then(() => {
      return this.#processMessage(wsMessage.data);
    });
  }
  /**
   * Call a reducer on your SpacetimeDB module.
   *
   * @param reducerName The name of the reducer to call
   * @param argsSerializer The arguments to pass to the reducer
   */
  callReducer(reducerName, argsBuffer, flags) {
    const message = ClientMessage.CallReducer({
      reducer: reducerName,
      args: argsBuffer,
      // The TypeScript SDK doesn't currently track `request_id`s,
      // so always use 0.
      requestId: 0,
      flags: callReducerFlagsToNumber(flags)
    });
    this.#sendMessage(message);
  }
  /**
   * Close the current connection.
   *
   * @example
   *
   * ```ts
   * const connection = DbConnection.builder().build();
   * connection.disconnect()
   * ```
   */
  disconnect() {
    this.wsPromise.then((wsResolved) => {
      if (wsResolved) {
        wsResolved.close();
      }
    });
  }
  on(eventName, callback) {
    this.#emitter.on(eventName, callback);
  }
  off(eventName, callback) {
    this.#emitter.off(eventName, callback);
  }
  onConnect(callback) {
    this.#emitter.on("connect", callback);
  }
  onDisconnect(callback) {
    this.#emitter.on("disconnect", callback);
  }
  onConnectError(callback) {
    this.#emitter.on("connectError", callback);
  }
  removeOnConnect(callback) {
    this.#emitter.off("connect", callback);
  }
  removeOnDisconnect(callback) {
    this.#emitter.off("disconnect", callback);
  }
  removeOnConnectError(callback) {
    this.#emitter.off("connectError", callback);
  }
  // Note: This is required to be public because it needs to be
  // called from the `RemoteReducers` class.
  onReducer(reducerName, callback) {
    this.#reducerEmitter.on(reducerName, callback);
  }
  // Note: This is required to be public because it needs to be
  // called from the `RemoteReducers` class.
  offReducer(reducerName, callback) {
    this.#reducerEmitter.off(reducerName, callback);
  }
};

// add_conversation_message_reducer.ts
var _cached_AddConversationMessage_type_value = null;
var AddConversationMessage = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_AddConversationMessage_type_value) return _cached_AddConversationMessage_type_value;
    _cached_AddConversationMessage_type_value = AlgebraicType.Product({ elements: [] });
    _cached_AddConversationMessage_type_value.value.elements.push(
      { name: "sender", algebraicType: AlgebraicType.String },
      { name: "message", algebraicType: AlgebraicType.String },
      { name: "generationContext", algebraicType: AlgebraicType.U64 }
    );
    return _cached_AddConversationMessage_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, AddConversationMessage.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, AddConversationMessage.getTypeScriptAlgebraicType());
  }
};

// client_connected_reducer.ts
var _cached_ClientConnected_type_value = null;
var ClientConnected = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_ClientConnected_type_value) return _cached_ClientConnected_type_value;
    _cached_ClientConnected_type_value = AlgebraicType.Product({ elements: [] });
    _cached_ClientConnected_type_value.value.elements.push();
    return _cached_ClientConnected_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, ClientConnected.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, ClientConnected.getTypeScriptAlgebraicType());
  }
};

// client_disconnected_reducer.ts
var _cached_ClientDisconnected_type_value = null;
var ClientDisconnected = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_ClientDisconnected_type_value) return _cached_ClientDisconnected_type_value;
    _cached_ClientDisconnected_type_value = AlgebraicType.Product({ elements: [] });
    _cached_ClientDisconnected_type_value.value.elements.push();
    return _cached_ClientDisconnected_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, ClientDisconnected.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, ClientDisconnected.getTypeScriptAlgebraicType());
  }
};

// get_recent_metrics_reducer.ts
var _cached_GetRecentMetrics_type_value = null;
var GetRecentMetrics = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_GetRecentMetrics_type_value) return _cached_GetRecentMetrics_type_value;
    _cached_GetRecentMetrics_type_value = AlgebraicType.Product({ elements: [] });
    _cached_GetRecentMetrics_type_value.value.elements.push(
      { name: "limit", algebraicType: AlgebraicType.U32 }
    );
    return _cached_GetRecentMetrics_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, GetRecentMetrics.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, GetRecentMetrics.getTypeScriptAlgebraicType());
  }
};

// log_training_event_reducer.ts
var _cached_LogTrainingEvent_type_value = null;
var LogTrainingEvent = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_LogTrainingEvent_type_value) return _cached_LogTrainingEvent_type_value;
    _cached_LogTrainingEvent_type_value = AlgebraicType.Product({ elements: [] });
    _cached_LogTrainingEvent_type_value.value.elements.push(
      { name: "generation", algebraicType: AlgebraicType.U64 },
      { name: "eventType", algebraicType: AlgebraicType.String },
      { name: "description", algebraicType: AlgebraicType.String }
    );
    return _cached_LogTrainingEvent_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, LogTrainingEvent.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, LogTrainingEvent.getTypeScriptAlgebraicType());
  }
};

// master_pattern_reducer.ts
var _cached_MasterPattern_type_value = null;
var MasterPattern = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_MasterPattern_type_value) return _cached_MasterPattern_type_value;
    _cached_MasterPattern_type_value = AlgebraicType.Product({ elements: [] });
    _cached_MasterPattern_type_value.value.elements.push(
      { name: "pattern", algebraicType: AlgebraicType.String },
      { name: "generation", algebraicType: AlgebraicType.U64 },
      { name: "finalLoss", algebraicType: AlgebraicType.F64 }
    );
    return _cached_MasterPattern_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, MasterPattern.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, MasterPattern.getTypeScriptAlgebraicType());
  }
};

// save_network_snapshot_reducer.ts
var _cached_SaveNetworkSnapshot_type_value = null;
var SaveNetworkSnapshot = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_SaveNetworkSnapshot_type_value) return _cached_SaveNetworkSnapshot_type_value;
    _cached_SaveNetworkSnapshot_type_value = AlgebraicType.Product({ elements: [] });
    _cached_SaveNetworkSnapshot_type_value.value.elements.push(
      { name: "generation", algebraicType: AlgebraicType.U64 },
      { name: "pattern", algebraicType: AlgebraicType.String },
      { name: "loss", algebraicType: AlgebraicType.F64 },
      { name: "weightsJson", algebraicType: AlgebraicType.String }
    );
    return _cached_SaveNetworkSnapshot_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, SaveNetworkSnapshot.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, SaveNetworkSnapshot.getTypeScriptAlgebraicType());
  }
};

// set_training_status_reducer.ts
var _cached_SetTrainingStatus_type_value = null;
var SetTrainingStatus = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_SetTrainingStatus_type_value) return _cached_SetTrainingStatus_type_value;
    _cached_SetTrainingStatus_type_value = AlgebraicType.Product({ elements: [] });
    _cached_SetTrainingStatus_type_value.value.elements.push(
      { name: "isTraining", algebraicType: AlgebraicType.Bool }
    );
    return _cached_SetTrainingStatus_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, SetTrainingStatus.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, SetTrainingStatus.getTypeScriptAlgebraicType());
  }
};

// start_pattern_reducer.ts
var _cached_StartPattern_type_value = null;
var StartPattern = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_StartPattern_type_value) return _cached_StartPattern_type_value;
    _cached_StartPattern_type_value = AlgebraicType.Product({ elements: [] });
    _cached_StartPattern_type_value.value.elements.push(
      { name: "pattern", algebraicType: AlgebraicType.String },
      { name: "generation", algebraicType: AlgebraicType.U64 }
    );
    return _cached_StartPattern_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, StartPattern.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, StartPattern.getTypeScriptAlgebraicType());
  }
};

// update_sage_state_reducer.ts
var _cached_UpdateSageState_type_value = null;
var UpdateSageState = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_UpdateSageState_type_value) return _cached_UpdateSageState_type_value;
    _cached_UpdateSageState_type_value = AlgebraicType.Product({ elements: [] });
    _cached_UpdateSageState_type_value.value.elements.push(
      { name: "generation", algebraicType: AlgebraicType.U64 },
      { name: "loss", algebraicType: AlgebraicType.F64 },
      { name: "pattern", algebraicType: AlgebraicType.String },
      { name: "complexity", algebraicType: AlgebraicType.F64 },
      { name: "diversity", algebraicType: AlgebraicType.F64 }
    );
    return _cached_UpdateSageState_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, UpdateSageState.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, UpdateSageState.getTypeScriptAlgebraicType());
  }
};

// conversations_table.ts
var ConversationsTableHandle = class {
  // phantom type to track the table name
  tableName;
  tableCache;
  constructor(tableCache) {
    this.tableCache = tableCache;
  }
  count() {
    return this.tableCache.count();
  }
  iter() {
    return this.tableCache.iter();
  }
  /**
   * Access to the `id` unique index on the table `conversations`,
   * which allows point queries on the field of the same name
   * via the [`ConversationsIdUnique.find`] method.
   *
   * Users are encouraged not to explicitly reference this type,
   * but to directly chain method calls,
   * like `ctx.db.conversations.id().find(...)`.
   *
   * Get a handle on the `id` unique index on the table `conversations`.
   */
  id = {
    // Find the subscribed row whose `id` column value is equal to `col_val`,
    // if such a row is present in the client cache.
    find: (col_val) => {
      for (let row of this.tableCache.iter()) {
        if (deepEqual(row.id, col_val)) {
          return row;
        }
      }
    }
  };
  onInsert = (cb) => {
    return this.tableCache.onInsert(cb);
  };
  removeOnInsert = (cb) => {
    return this.tableCache.removeOnInsert(cb);
  };
  onDelete = (cb) => {
    return this.tableCache.onDelete(cb);
  };
  removeOnDelete = (cb) => {
    return this.tableCache.removeOnDelete(cb);
  };
  // Updates are only defined for tables with primary keys.
  onUpdate = (cb) => {
    return this.tableCache.onUpdate(cb);
  };
  removeOnUpdate = (cb) => {
    return this.tableCache.removeOnUpdate(cb);
  };
};

// network_snapshots_table.ts
var NetworkSnapshotsTableHandle = class {
  // phantom type to track the table name
  tableName;
  tableCache;
  constructor(tableCache) {
    this.tableCache = tableCache;
  }
  count() {
    return this.tableCache.count();
  }
  iter() {
    return this.tableCache.iter();
  }
  /**
   * Access to the `id` unique index on the table `network_snapshots`,
   * which allows point queries on the field of the same name
   * via the [`NetworkSnapshotsIdUnique.find`] method.
   *
   * Users are encouraged not to explicitly reference this type,
   * but to directly chain method calls,
   * like `ctx.db.networkSnapshots.id().find(...)`.
   *
   * Get a handle on the `id` unique index on the table `network_snapshots`.
   */
  id = {
    // Find the subscribed row whose `id` column value is equal to `col_val`,
    // if such a row is present in the client cache.
    find: (col_val) => {
      for (let row of this.tableCache.iter()) {
        if (deepEqual(row.id, col_val)) {
          return row;
        }
      }
    }
  };
  onInsert = (cb) => {
    return this.tableCache.onInsert(cb);
  };
  removeOnInsert = (cb) => {
    return this.tableCache.removeOnInsert(cb);
  };
  onDelete = (cb) => {
    return this.tableCache.onDelete(cb);
  };
  removeOnDelete = (cb) => {
    return this.tableCache.removeOnDelete(cb);
  };
  // Updates are only defined for tables with primary keys.
  onUpdate = (cb) => {
    return this.tableCache.onUpdate(cb);
  };
  removeOnUpdate = (cb) => {
    return this.tableCache.removeOnUpdate(cb);
  };
};

// pattern_progress_table.ts
var PatternProgressTableHandle = class {
  // phantom type to track the table name
  tableName;
  tableCache;
  constructor(tableCache) {
    this.tableCache = tableCache;
  }
  count() {
    return this.tableCache.count();
  }
  iter() {
    return this.tableCache.iter();
  }
  /**
   * Access to the `id` unique index on the table `pattern_progress`,
   * which allows point queries on the field of the same name
   * via the [`PatternProgressIdUnique.find`] method.
   *
   * Users are encouraged not to explicitly reference this type,
   * but to directly chain method calls,
   * like `ctx.db.patternProgress.id().find(...)`.
   *
   * Get a handle on the `id` unique index on the table `pattern_progress`.
   */
  id = {
    // Find the subscribed row whose `id` column value is equal to `col_val`,
    // if such a row is present in the client cache.
    find: (col_val) => {
      for (let row of this.tableCache.iter()) {
        if (deepEqual(row.id, col_val)) {
          return row;
        }
      }
    }
  };
  onInsert = (cb) => {
    return this.tableCache.onInsert(cb);
  };
  removeOnInsert = (cb) => {
    return this.tableCache.removeOnInsert(cb);
  };
  onDelete = (cb) => {
    return this.tableCache.onDelete(cb);
  };
  removeOnDelete = (cb) => {
    return this.tableCache.removeOnDelete(cb);
  };
  // Updates are only defined for tables with primary keys.
  onUpdate = (cb) => {
    return this.tableCache.onUpdate(cb);
  };
  removeOnUpdate = (cb) => {
    return this.tableCache.removeOnUpdate(cb);
  };
};

// sage_state_table.ts
var SageStateTableHandle = class {
  // phantom type to track the table name
  tableName;
  tableCache;
  constructor(tableCache) {
    this.tableCache = tableCache;
  }
  count() {
    return this.tableCache.count();
  }
  iter() {
    return this.tableCache.iter();
  }
  /**
   * Access to the `id` unique index on the table `sage_state`,
   * which allows point queries on the field of the same name
   * via the [`SageStateIdUnique.find`] method.
   *
   * Users are encouraged not to explicitly reference this type,
   * but to directly chain method calls,
   * like `ctx.db.sageState.id().find(...)`.
   *
   * Get a handle on the `id` unique index on the table `sage_state`.
   */
  id = {
    // Find the subscribed row whose `id` column value is equal to `col_val`,
    // if such a row is present in the client cache.
    find: (col_val) => {
      for (let row of this.tableCache.iter()) {
        if (deepEqual(row.id, col_val)) {
          return row;
        }
      }
    }
  };
  onInsert = (cb) => {
    return this.tableCache.onInsert(cb);
  };
  removeOnInsert = (cb) => {
    return this.tableCache.removeOnInsert(cb);
  };
  onDelete = (cb) => {
    return this.tableCache.onDelete(cb);
  };
  removeOnDelete = (cb) => {
    return this.tableCache.removeOnDelete(cb);
  };
  // Updates are only defined for tables with primary keys.
  onUpdate = (cb) => {
    return this.tableCache.onUpdate(cb);
  };
  removeOnUpdate = (cb) => {
    return this.tableCache.removeOnUpdate(cb);
  };
};

// training_events_table.ts
var TrainingEventsTableHandle = class {
  // phantom type to track the table name
  tableName;
  tableCache;
  constructor(tableCache) {
    this.tableCache = tableCache;
  }
  count() {
    return this.tableCache.count();
  }
  iter() {
    return this.tableCache.iter();
  }
  /**
   * Access to the `id` unique index on the table `training_events`,
   * which allows point queries on the field of the same name
   * via the [`TrainingEventsIdUnique.find`] method.
   *
   * Users are encouraged not to explicitly reference this type,
   * but to directly chain method calls,
   * like `ctx.db.trainingEvents.id().find(...)`.
   *
   * Get a handle on the `id` unique index on the table `training_events`.
   */
  id = {
    // Find the subscribed row whose `id` column value is equal to `col_val`,
    // if such a row is present in the client cache.
    find: (col_val) => {
      for (let row of this.tableCache.iter()) {
        if (deepEqual(row.id, col_val)) {
          return row;
        }
      }
    }
  };
  onInsert = (cb) => {
    return this.tableCache.onInsert(cb);
  };
  removeOnInsert = (cb) => {
    return this.tableCache.removeOnInsert(cb);
  };
  onDelete = (cb) => {
    return this.tableCache.onDelete(cb);
  };
  removeOnDelete = (cb) => {
    return this.tableCache.removeOnDelete(cb);
  };
  // Updates are only defined for tables with primary keys.
  onUpdate = (cb) => {
    return this.tableCache.onUpdate(cb);
  };
  removeOnUpdate = (cb) => {
    return this.tableCache.removeOnUpdate(cb);
  };
};

// training_metrics_table.ts
var TrainingMetricsTableHandle = class {
  // phantom type to track the table name
  tableName;
  tableCache;
  constructor(tableCache) {
    this.tableCache = tableCache;
  }
  count() {
    return this.tableCache.count();
  }
  iter() {
    return this.tableCache.iter();
  }
  /**
   * Access to the `id` unique index on the table `training_metrics`,
   * which allows point queries on the field of the same name
   * via the [`TrainingMetricsIdUnique.find`] method.
   *
   * Users are encouraged not to explicitly reference this type,
   * but to directly chain method calls,
   * like `ctx.db.trainingMetrics.id().find(...)`.
   *
   * Get a handle on the `id` unique index on the table `training_metrics`.
   */
  id = {
    // Find the subscribed row whose `id` column value is equal to `col_val`,
    // if such a row is present in the client cache.
    find: (col_val) => {
      for (let row of this.tableCache.iter()) {
        if (deepEqual(row.id, col_val)) {
          return row;
        }
      }
    }
  };
  onInsert = (cb) => {
    return this.tableCache.onInsert(cb);
  };
  removeOnInsert = (cb) => {
    return this.tableCache.removeOnInsert(cb);
  };
  onDelete = (cb) => {
    return this.tableCache.onDelete(cb);
  };
  removeOnDelete = (cb) => {
    return this.tableCache.removeOnDelete(cb);
  };
  // Updates are only defined for tables with primary keys.
  onUpdate = (cb) => {
    return this.tableCache.onUpdate(cb);
  };
  removeOnUpdate = (cb) => {
    return this.tableCache.removeOnUpdate(cb);
  };
};

// conversation_type.ts
var _cached_Conversation_type_value = null;
var Conversation = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_Conversation_type_value) return _cached_Conversation_type_value;
    _cached_Conversation_type_value = AlgebraicType.Product({ elements: [] });
    _cached_Conversation_type_value.value.elements.push(
      { name: "id", algebraicType: AlgebraicType.U64 },
      { name: "sender", algebraicType: AlgebraicType.String },
      { name: "message", algebraicType: AlgebraicType.String },
      { name: "generationContext", algebraicType: AlgebraicType.U64 },
      { name: "timestamp", algebraicType: AlgebraicType.createTimestampType() }
    );
    return _cached_Conversation_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, Conversation.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, Conversation.getTypeScriptAlgebraicType());
  }
};

// network_snapshot_type.ts
var _cached_NetworkSnapshot_type_value = null;
var NetworkSnapshot = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_NetworkSnapshot_type_value) return _cached_NetworkSnapshot_type_value;
    _cached_NetworkSnapshot_type_value = AlgebraicType.Product({ elements: [] });
    _cached_NetworkSnapshot_type_value.value.elements.push(
      { name: "id", algebraicType: AlgebraicType.U64 },
      { name: "generation", algebraicType: AlgebraicType.U64 },
      { name: "pattern", algebraicType: AlgebraicType.String },
      { name: "loss", algebraicType: AlgebraicType.F64 },
      { name: "weightsJson", algebraicType: AlgebraicType.String },
      { name: "timestamp", algebraicType: AlgebraicType.createTimestampType() }
    );
    return _cached_NetworkSnapshot_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, NetworkSnapshot.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, NetworkSnapshot.getTypeScriptAlgebraicType());
  }
};

// pattern_progress_type.ts
var _cached_PatternProgress_type_value = null;
var PatternProgress = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_PatternProgress_type_value) return _cached_PatternProgress_type_value;
    _cached_PatternProgress_type_value = AlgebraicType.Product({ elements: [] });
    _cached_PatternProgress_type_value.value.elements.push(
      { name: "id", algebraicType: AlgebraicType.U64 },
      { name: "pattern", algebraicType: AlgebraicType.String },
      { name: "startGeneration", algebraicType: AlgebraicType.U64 },
      { name: "masteredGeneration", algebraicType: AlgebraicType.createOptionType(AlgebraicType.U64) },
      { name: "bestLoss", algebraicType: AlgebraicType.F64 },
      { name: "isMastered", algebraicType: AlgebraicType.Bool },
      { name: "startedAt", algebraicType: AlgebraicType.createTimestampType() },
      { name: "masteredAt", algebraicType: AlgebraicType.createOptionType(AlgebraicType.createTimestampType()) }
    );
    return _cached_PatternProgress_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, PatternProgress.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, PatternProgress.getTypeScriptAlgebraicType());
  }
};

// sage_state_type.ts
var _cached_SageState_type_value = null;
var SageState = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_SageState_type_value) return _cached_SageState_type_value;
    _cached_SageState_type_value = AlgebraicType.Product({ elements: [] });
    _cached_SageState_type_value.value.elements.push(
      { name: "id", algebraicType: AlgebraicType.U64 },
      { name: "generation", algebraicType: AlgebraicType.U64 },
      { name: "currentLoss", algebraicType: AlgebraicType.F64 },
      { name: "currentPattern", algebraicType: AlgebraicType.String },
      { name: "complexity", algebraicType: AlgebraicType.F64 },
      { name: "diversity", algebraicType: AlgebraicType.F64 },
      { name: "isTraining", algebraicType: AlgebraicType.Bool },
      { name: "updatedAt", algebraicType: AlgebraicType.createTimestampType() }
    );
    return _cached_SageState_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, SageState.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, SageState.getTypeScriptAlgebraicType());
  }
};

// training_event_type.ts
var _cached_TrainingEvent_type_value = null;
var TrainingEvent = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_TrainingEvent_type_value) return _cached_TrainingEvent_type_value;
    _cached_TrainingEvent_type_value = AlgebraicType.Product({ elements: [] });
    _cached_TrainingEvent_type_value.value.elements.push(
      { name: "id", algebraicType: AlgebraicType.U64 },
      { name: "generation", algebraicType: AlgebraicType.U64 },
      { name: "eventType", algebraicType: AlgebraicType.String },
      { name: "description", algebraicType: AlgebraicType.String },
      { name: "timestamp", algebraicType: AlgebraicType.createTimestampType() }
    );
    return _cached_TrainingEvent_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, TrainingEvent.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, TrainingEvent.getTypeScriptAlgebraicType());
  }
};

// training_metrics_type.ts
var _cached_TrainingMetrics_type_value = null;
var TrainingMetrics = {
  /**
  * A function which returns this type represented as an AlgebraicType.
  * This function is derived from the AlgebraicType used to generate this type.
  */
  getTypeScriptAlgebraicType() {
    if (_cached_TrainingMetrics_type_value) return _cached_TrainingMetrics_type_value;
    _cached_TrainingMetrics_type_value = AlgebraicType.Product({ elements: [] });
    _cached_TrainingMetrics_type_value.value.elements.push(
      { name: "id", algebraicType: AlgebraicType.U64 },
      { name: "generation", algebraicType: AlgebraicType.U64 },
      { name: "loss", algebraicType: AlgebraicType.F64 },
      { name: "complexity", algebraicType: AlgebraicType.F64 },
      { name: "diversity", algebraicType: AlgebraicType.F64 },
      { name: "pattern", algebraicType: AlgebraicType.String },
      { name: "timestamp", algebraicType: AlgebraicType.createTimestampType() }
    );
    return _cached_TrainingMetrics_type_value;
  },
  serialize(writer, value) {
    AlgebraicType.serializeValue(writer, TrainingMetrics.getTypeScriptAlgebraicType(), value);
  },
  deserialize(reader) {
    return AlgebraicType.deserializeValue(reader, TrainingMetrics.getTypeScriptAlgebraicType());
  }
};

// index.ts
var REMOTE_MODULE = {
  tables: {
    conversations: {
      tableName: "conversations",
      rowType: Conversation.getTypeScriptAlgebraicType(),
      primaryKey: "id",
      primaryKeyInfo: {
        colName: "id",
        colType: Conversation.getTypeScriptAlgebraicType().value.elements[0].algebraicType
      }
    },
    network_snapshots: {
      tableName: "network_snapshots",
      rowType: NetworkSnapshot.getTypeScriptAlgebraicType(),
      primaryKey: "id",
      primaryKeyInfo: {
        colName: "id",
        colType: NetworkSnapshot.getTypeScriptAlgebraicType().value.elements[0].algebraicType
      }
    },
    pattern_progress: {
      tableName: "pattern_progress",
      rowType: PatternProgress.getTypeScriptAlgebraicType(),
      primaryKey: "id",
      primaryKeyInfo: {
        colName: "id",
        colType: PatternProgress.getTypeScriptAlgebraicType().value.elements[0].algebraicType
      }
    },
    sage_state: {
      tableName: "sage_state",
      rowType: SageState.getTypeScriptAlgebraicType(),
      primaryKey: "id",
      primaryKeyInfo: {
        colName: "id",
        colType: SageState.getTypeScriptAlgebraicType().value.elements[0].algebraicType
      }
    },
    training_events: {
      tableName: "training_events",
      rowType: TrainingEvent.getTypeScriptAlgebraicType(),
      primaryKey: "id",
      primaryKeyInfo: {
        colName: "id",
        colType: TrainingEvent.getTypeScriptAlgebraicType().value.elements[0].algebraicType
      }
    },
    training_metrics: {
      tableName: "training_metrics",
      rowType: TrainingMetrics.getTypeScriptAlgebraicType(),
      primaryKey: "id",
      primaryKeyInfo: {
        colName: "id",
        colType: TrainingMetrics.getTypeScriptAlgebraicType().value.elements[0].algebraicType
      }
    }
  },
  reducers: {
    add_conversation_message: {
      reducerName: "add_conversation_message",
      argsType: AddConversationMessage.getTypeScriptAlgebraicType()
    },
    client_connected: {
      reducerName: "client_connected",
      argsType: ClientConnected.getTypeScriptAlgebraicType()
    },
    client_disconnected: {
      reducerName: "client_disconnected",
      argsType: ClientDisconnected.getTypeScriptAlgebraicType()
    },
    get_recent_metrics: {
      reducerName: "get_recent_metrics",
      argsType: GetRecentMetrics.getTypeScriptAlgebraicType()
    },
    log_training_event: {
      reducerName: "log_training_event",
      argsType: LogTrainingEvent.getTypeScriptAlgebraicType()
    },
    master_pattern: {
      reducerName: "master_pattern",
      argsType: MasterPattern.getTypeScriptAlgebraicType()
    },
    save_network_snapshot: {
      reducerName: "save_network_snapshot",
      argsType: SaveNetworkSnapshot.getTypeScriptAlgebraicType()
    },
    set_training_status: {
      reducerName: "set_training_status",
      argsType: SetTrainingStatus.getTypeScriptAlgebraicType()
    },
    start_pattern: {
      reducerName: "start_pattern",
      argsType: StartPattern.getTypeScriptAlgebraicType()
    },
    update_sage_state: {
      reducerName: "update_sage_state",
      argsType: UpdateSageState.getTypeScriptAlgebraicType()
    }
  },
  versionInfo: {
    cliVersion: "1.6.0"
  },
  // Constructors which are used by the DbConnectionImpl to
  // extract type information from the generated RemoteModule.
  //
  // NOTE: This is not strictly necessary for `eventContextConstructor` because
  // all we do is build a TypeScript object which we could have done inside the
  // SDK, but if in the future we wanted to create a class this would be
  // necessary because classes have methods, so we'll keep it.
  eventContextConstructor: (imp, event) => {
    return {
      ...imp,
      event
    };
  },
  dbViewConstructor: (imp) => {
    return new RemoteTables(imp);
  },
  reducersConstructor: (imp, setReducerFlags) => {
    return new RemoteReducers(imp, setReducerFlags);
  },
  setReducerFlagsConstructor: () => {
    return new SetReducerFlags();
  }
};
var RemoteReducers = class {
  constructor(connection2, setCallReducerFlags) {
    this.connection = connection2;
    this.setCallReducerFlags = setCallReducerFlags;
  }
  addConversationMessage(sender, message, generationContext) {
    const __args = { sender, message, generationContext };
    let __writer = new BinaryWriter(1024);
    AddConversationMessage.serialize(__writer, __args);
    let __argsBuffer = __writer.getBuffer();
    this.connection.callReducer("add_conversation_message", __argsBuffer, this.setCallReducerFlags.addConversationMessageFlags);
  }
  onAddConversationMessage(callback) {
    this.connection.onReducer("add_conversation_message", callback);
  }
  removeOnAddConversationMessage(callback) {
    this.connection.offReducer("add_conversation_message", callback);
  }
  onClientConnected(callback) {
    this.connection.onReducer("client_connected", callback);
  }
  removeOnClientConnected(callback) {
    this.connection.offReducer("client_connected", callback);
  }
  onClientDisconnected(callback) {
    this.connection.onReducer("client_disconnected", callback);
  }
  removeOnClientDisconnected(callback) {
    this.connection.offReducer("client_disconnected", callback);
  }
  getRecentMetrics(limit) {
    const __args = { limit };
    let __writer = new BinaryWriter(1024);
    GetRecentMetrics.serialize(__writer, __args);
    let __argsBuffer = __writer.getBuffer();
    this.connection.callReducer("get_recent_metrics", __argsBuffer, this.setCallReducerFlags.getRecentMetricsFlags);
  }
  onGetRecentMetrics(callback) {
    this.connection.onReducer("get_recent_metrics", callback);
  }
  removeOnGetRecentMetrics(callback) {
    this.connection.offReducer("get_recent_metrics", callback);
  }
  logTrainingEvent(generation, eventType, description) {
    const __args = { generation, eventType, description };
    let __writer = new BinaryWriter(1024);
    LogTrainingEvent.serialize(__writer, __args);
    let __argsBuffer = __writer.getBuffer();
    this.connection.callReducer("log_training_event", __argsBuffer, this.setCallReducerFlags.logTrainingEventFlags);
  }
  onLogTrainingEvent(callback) {
    this.connection.onReducer("log_training_event", callback);
  }
  removeOnLogTrainingEvent(callback) {
    this.connection.offReducer("log_training_event", callback);
  }
  masterPattern(pattern, generation, finalLoss) {
    const __args = { pattern, generation, finalLoss };
    let __writer = new BinaryWriter(1024);
    MasterPattern.serialize(__writer, __args);
    let __argsBuffer = __writer.getBuffer();
    this.connection.callReducer("master_pattern", __argsBuffer, this.setCallReducerFlags.masterPatternFlags);
  }
  onMasterPattern(callback) {
    this.connection.onReducer("master_pattern", callback);
  }
  removeOnMasterPattern(callback) {
    this.connection.offReducer("master_pattern", callback);
  }
  saveNetworkSnapshot(generation, pattern, loss, weightsJson) {
    const __args = { generation, pattern, loss, weightsJson };
    let __writer = new BinaryWriter(1024);
    SaveNetworkSnapshot.serialize(__writer, __args);
    let __argsBuffer = __writer.getBuffer();
    this.connection.callReducer("save_network_snapshot", __argsBuffer, this.setCallReducerFlags.saveNetworkSnapshotFlags);
  }
  onSaveNetworkSnapshot(callback) {
    this.connection.onReducer("save_network_snapshot", callback);
  }
  removeOnSaveNetworkSnapshot(callback) {
    this.connection.offReducer("save_network_snapshot", callback);
  }
  setTrainingStatus(isTraining) {
    const __args = { isTraining };
    let __writer = new BinaryWriter(1024);
    SetTrainingStatus.serialize(__writer, __args);
    let __argsBuffer = __writer.getBuffer();
    this.connection.callReducer("set_training_status", __argsBuffer, this.setCallReducerFlags.setTrainingStatusFlags);
  }
  onSetTrainingStatus(callback) {
    this.connection.onReducer("set_training_status", callback);
  }
  removeOnSetTrainingStatus(callback) {
    this.connection.offReducer("set_training_status", callback);
  }
  startPattern(pattern, generation) {
    const __args = { pattern, generation };
    let __writer = new BinaryWriter(1024);
    StartPattern.serialize(__writer, __args);
    let __argsBuffer = __writer.getBuffer();
    this.connection.callReducer("start_pattern", __argsBuffer, this.setCallReducerFlags.startPatternFlags);
  }
  onStartPattern(callback) {
    this.connection.onReducer("start_pattern", callback);
  }
  removeOnStartPattern(callback) {
    this.connection.offReducer("start_pattern", callback);
  }
  updateSageState(generation, loss, pattern, complexity, diversity) {
    const __args = { generation, loss, pattern, complexity, diversity };
    let __writer = new BinaryWriter(1024);
    UpdateSageState.serialize(__writer, __args);
    let __argsBuffer = __writer.getBuffer();
    this.connection.callReducer("update_sage_state", __argsBuffer, this.setCallReducerFlags.updateSageStateFlags);
  }
  onUpdateSageState(callback) {
    this.connection.onReducer("update_sage_state", callback);
  }
  removeOnUpdateSageState(callback) {
    this.connection.offReducer("update_sage_state", callback);
  }
};
var SetReducerFlags = class {
  addConversationMessageFlags = "FullUpdate";
  addConversationMessage(flags) {
    this.addConversationMessageFlags = flags;
  }
  getRecentMetricsFlags = "FullUpdate";
  getRecentMetrics(flags) {
    this.getRecentMetricsFlags = flags;
  }
  logTrainingEventFlags = "FullUpdate";
  logTrainingEvent(flags) {
    this.logTrainingEventFlags = flags;
  }
  masterPatternFlags = "FullUpdate";
  masterPattern(flags) {
    this.masterPatternFlags = flags;
  }
  saveNetworkSnapshotFlags = "FullUpdate";
  saveNetworkSnapshot(flags) {
    this.saveNetworkSnapshotFlags = flags;
  }
  setTrainingStatusFlags = "FullUpdate";
  setTrainingStatus(flags) {
    this.setTrainingStatusFlags = flags;
  }
  startPatternFlags = "FullUpdate";
  startPattern(flags) {
    this.startPatternFlags = flags;
  }
  updateSageStateFlags = "FullUpdate";
  updateSageState(flags) {
    this.updateSageStateFlags = flags;
  }
};
var RemoteTables = class {
  constructor(connection2) {
    this.connection = connection2;
  }
  get conversations() {
    return new ConversationsTableHandle(this.connection.clientCache.getOrCreateTable(REMOTE_MODULE.tables.conversations));
  }
  get networkSnapshots() {
    return new NetworkSnapshotsTableHandle(this.connection.clientCache.getOrCreateTable(REMOTE_MODULE.tables.network_snapshots));
  }
  get patternProgress() {
    return new PatternProgressTableHandle(this.connection.clientCache.getOrCreateTable(REMOTE_MODULE.tables.pattern_progress));
  }
  get sageState() {
    return new SageStateTableHandle(this.connection.clientCache.getOrCreateTable(REMOTE_MODULE.tables.sage_state));
  }
  get trainingEvents() {
    return new TrainingEventsTableHandle(this.connection.clientCache.getOrCreateTable(REMOTE_MODULE.tables.training_events));
  }
  get trainingMetrics() {
    return new TrainingMetricsTableHandle(this.connection.clientCache.getOrCreateTable(REMOTE_MODULE.tables.training_metrics));
  }
};
var SubscriptionBuilder = class extends SubscriptionBuilderImpl {
};
var DbConnection = class extends DbConnectionImpl {
  static builder = () => {
    return new DbConnectionBuilder(REMOTE_MODULE, (imp) => imp);
  };
  subscriptionBuilder = () => {
    return new SubscriptionBuilder(this);
  };
};

// app.ts
var connection = null;
function getLossColor(loss) {
  if (loss < 0.05) return "green";
  if (loss < 0.1) return "yellow";
  return "red";
}
function updateCurrentState() {
  if (!connection) return;
  const states = Array.from(connection.db.sageState.iter());
  const el = document.getElementById("current-state");
  if (!el) return;
  if (states.length === 0) {
    el.innerHTML = '<div class="loading">No data yet...</div>';
    return;
  }
  const state = states.reduce(
    (max, curr) => curr.generation > max.generation ? curr : max
  );
  const lossColor = getLossColor(state.currentLoss);
  const statusClass = state.isTraining ? "active" : "paused";
  const statusText = state.isTraining ? "\u{1F9EC} Active" : "\u23F8 Paused";
  el.innerHTML = `
        <div class="metric">
            <span class="label">Generation:</span>
            <span class="value cyan">${state.generation}</span>
        </div>
        <div class="metric">
            <span class="label">Loss:</span>
            <span class="value ${lossColor}">${state.currentLoss.toFixed(4)}</span>
        </div>
        <div class="metric">
            <span class="label">Pattern:</span>
            <span class="value">${state.currentPattern}</span>
        </div>
        <div class="metric">
            <span class="label">Complexity:</span>
            <span class="value">${state.complexity.toFixed(3)}</span>
        </div>
        <div class="metric">
            <span class="label">Diversity:</span>
            <span class="value">${state.diversity.toFixed(3)}</span>
        </div>
        <div class="metric">
            <span class="label">Training:</span>
            <span class="status ${statusClass}">${statusText}</span>
        </div>
    `;
}
function updatePatternProgress() {
  if (!connection) return;
  const patterns = Array.from(connection.db.patternProgress.iter());
  const el = document.getElementById("pattern-progress");
  if (!el) return;
  if (patterns.length === 0) {
    el.innerHTML = '<div class="loading">No patterns yet...</div>';
    return;
  }
  el.innerHTML = patterns.map((p) => {
    const icon = p.isMastered ? "\u2713" : "\u25CB";
    const color = p.isMastered ? "green" : "yellow";
    return `
            <div class="metric">
                <span class="value ${color}">${icon} ${p.pattern}</span>
                <span class="value">${p.bestLoss.toFixed(4)}</span>
            </div>
        `;
  }).join("");
}
function updateRecentMetrics() {
  if (!connection) return;
  const metrics = Array.from(connection.db.trainingMetrics.iter()).sort((a, b) => Number(b.generation - a.generation)).slice(0, 8).reverse();
  const el = document.getElementById("recent-metrics");
  if (!el) return;
  if (metrics.length === 0) {
    el.innerHTML = '<div class="loading">No metrics yet...</div>';
    return;
  }
  el.innerHTML = metrics.map((m) => {
    const lossColor = getLossColor(m.loss);
    return `
            <div class="metric">
                <span class="label">Gen ${m.generation}:</span>
                <span class="value ${lossColor}">${m.loss.toFixed(4)}</span>
            </div>
        `;
  }).join("");
}
function updateTrainingEvents() {
  if (!connection) return;
  const events = Array.from(connection.db.trainingEvents.iter()).sort((a, b) => Number(b.generation - a.generation)).slice(0, 10).reverse();
  const el = document.getElementById("training-events");
  if (!el) return;
  if (events.length === 0) {
    el.innerHTML = '<div class="loading">No events yet...</div>';
    return;
  }
  el.innerHTML = events.map((e) => `
        <div class="event">
            <strong>Gen ${e.generation}:</strong> ${e.description}
        </div>
    `).join("");
}
function updateConversations() {
  if (!connection) return;
  const conversations = Array.from(connection.db.conversations.iter()).sort((a, b) => Number(b.generationContext - a.generationContext)).slice(0, 10).reverse();
  const el = document.getElementById("conversations");
  if (!el) return;
  if (conversations.length === 0) {
    el.innerHTML = '<div class="loading">No conversations yet...</div>';
    return;
  }
  el.innerHTML = conversations.map((c) => `
        <div class="conversation">
            <div class="sender ${c.sender.toLowerCase()}">${c.sender} <span class="gen">(gen ${c.generationContext})</span></div>
            <div class="message">${c.message}</div>
        </div>
    `).join("");
}
function updateAll() {
  updateCurrentState();
  updatePatternProgress();
  updateRecentMetrics();
  updateTrainingEvents();
  updateConversations();
}
function initDashboard() {
  console.log("Connecting to SpacetimeDB...");
  connection = DbConnection.builder().withUri("ws://127.0.0.1:4000").withModuleName("sage-db").onConnect((ctx, identity, token) => {
    console.log("Connected to SpacetimeDB!", identity);
    connection.subscriptionBuilder().onApplied((ctx2) => {
      console.log("Subscription applied, updating UI...");
      updateAll();
    }).subscribe(`
                    SELECT * FROM sage_state;
                    SELECT * FROM pattern_progress;
                    SELECT * FROM training_metrics;
                    SELECT * FROM training_events;
                    SELECT * FROM conversations;
                `);
  }).onError((ctx, err) => {
    console.error("SpacetimeDB error:", err);
    document.querySelectorAll(".loading").forEach((el) => {
      el.textContent = "Connection error. Is SpacetimeDB running?";
      el.classList.remove("pulse");
    });
  }).build();
  connection.db.sageState.onInsert(() => updateCurrentState());
  connection.db.sageState.onUpdate(() => updateCurrentState());
  connection.db.patternProgress.onInsert(() => updatePatternProgress());
  connection.db.patternProgress.onUpdate(() => updatePatternProgress());
  connection.db.trainingMetrics.onInsert(() => updateRecentMetrics());
  connection.db.trainingEvents.onInsert(() => updateTrainingEvents());
  connection.db.conversations.onInsert(() => updateConversations());
}
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initDashboard);
} else {
  initDashboard();
}
export {
  initDashboard
};
