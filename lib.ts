import { initialize, sendCommand } from './index'

import EventEmitter from 'events'

export class RodioBackend extends EventEmitter {
    #eventSender: Object

    constructor() {
        super()

        const ret = initialize()
        this.#eventSender = ret

        if (!this.#eventSender || typeof ret !== 'object') {
            throw new Error("Failed to initialize rodio backend")
        }

        this.on('timeUpdate', (pos) => {
            if (this.totalDuration && pos >= this.totalDuration - 400) {
                this.emit('ended')
            }
        })
    }

    private totalDuration: number | undefined = undefined

    public async setSrc(path: string) {
        const duration = await this.sendCommandAsync("SET_SRC", path)
        this.totalDuration = duration
        this.emit('loaded')
    }

    private sendCommandAsync(command: string, args?: string) {
        return new Promise<number>((resolve, reject) => {
            sendCommand(command, args ?? "", this.#eventSender, (err, ret) => {
                if (err) {
                    reject(err)
                }
                resolve(ret)
            })
        })
    }

    public async play() {
        await this.sendCommandAsync("PLAY")
        this.initializePositionListener()
        this.emit('play')
    }

    public async pause() {
        await this.sendCommandAsync("PAUSE")
        this.clearInterval()
        this.emit('pause')
    }

    public async stop() {
        await this.sendCommandAsync("STOP")
        this.clearInterval()
        this.emit('stop')
    }

    public setVolume(volume: number) {
        return this.sendCommandAsync("SET_VOLUME", Math.max(Math.min(volume, 1), 0).toString())
    }

    public getVolume() {
        return this.sendCommandAsync("GET_VOLUME")
    }

    public getPosition() {
        return this.sendCommandAsync("GET_POSITION")
    }

    public seek(pos: number) {
        return this.sendCommandAsync("SEEK", pos.toString())
    }

    private interval: ReturnType<typeof setInterval> | undefined

    private clearInterval() {
        if (this.interval) {
            clearInterval(this.interval)
            this.interval = undefined
        }
    }

    private initializePositionListener() {
        this.interval = setInterval(async () => {
            const pos = await this.getPosition()
            this.emit('timeUpdate', pos)
        }, 1000)
    }
}