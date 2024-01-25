"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __classPrivateFieldSet = (this && this.__classPrivateFieldSet) || function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet = (this && this.__classPrivateFieldGet) || function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var _RodioBackend_eventSender;
Object.defineProperty(exports, "__esModule", { value: true });
exports.RodioBackend = void 0;
const index_1 = require("./index");
const events_1 = __importDefault(require("events"));
class RodioBackend extends events_1.default {
    constructor() {
        super();
        _RodioBackend_eventSender.set(this, void 0);
        this.totalDuration = undefined;
        const ret = (0, index_1.initialize)();
        __classPrivateFieldSet(this, _RodioBackend_eventSender, ret, "f");
        if (!__classPrivateFieldGet(this, _RodioBackend_eventSender, "f") || typeof ret !== 'object') {
            throw new Error("Failed to initialize rodio backend");
        }
        this.on('timeUpdate', (pos) => {
            if (this.totalDuration && pos >= this.totalDuration - 400) {
                this.emit('ended');
            }
        });
    }
    setSrc(path) {
        return __awaiter(this, void 0, void 0, function* () {
            const duration = yield this.sendCommandAsync("SET_SRC", path);
            this.totalDuration = duration;
            this.emit('loaded');
        });
    }
    sendCommandAsync(command, args) {
        return new Promise((resolve, reject) => {
            (0, index_1.sendCommand)(command, args !== null && args !== void 0 ? args : "", __classPrivateFieldGet(this, _RodioBackend_eventSender, "f"), (err, ret) => {
                if (err) {
                    reject(err);
                }
                resolve(ret);
            });
        });
    }
    play() {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.sendCommandAsync("PLAY");
            this.initializePositionListener();
            this.emit('play');
        });
    }
    pause() {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.sendCommandAsync("PAUSE");
            this.clearInterval();
            this.emit('pause');
        });
    }
    stop() {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.sendCommandAsync("STOP");
            this.clearInterval();
            this.emit('stop');
        });
    }
    setVolume(volume) {
        return this.sendCommandAsync("SET_VOLUME", volume.toString());
    }
    getVolume() {
        return this.sendCommandAsync("GET_VOLUME");
    }
    getPosition() {
        return this.sendCommandAsync("GET_POSITION");
    }
    seek(pos) {
        return this.sendCommandAsync("SEEK", pos.toString());
    }
    clearInterval() {
        if (this.interval) {
            clearInterval(this.interval);
            this.interval = undefined;
        }
    }
    initializePositionListener() {
        this.interval = setInterval(() => __awaiter(this, void 0, void 0, function* () {
            const pos = yield this.getPosition();
            this.emit('timeUpdate', pos);
        }), 1000);
    }
}
exports.RodioBackend = RodioBackend;
_RodioBackend_eventSender = new WeakMap();
