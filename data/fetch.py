#!/usr/bin/env python3

import json
import subprocess


def run(args, **kwargs):
    return subprocess.check_output(args, universal_newlines=True, **kwargs)


def main():
    print(json.dumps({
        'nproc': get_nproc(),
        'uptime': get_uptime(),
        'memory': get_memory_info(),
        'disks': get_disks(),
        'network': get_traffic()
    }))


def get_nproc():
    try:
        return int(run('nproc'))
    except:
        return -1


def get_uptime():
    try:
        uptime = run('uptime').split(':')[-1]
        tokens = [float(token.strip()) for token in uptime.split(',')]
        return tokens
    except:
        return []


def get_memory_info():
    try:
        memory = run(['head', '-n', '4', '/proc/meminfo'])
        lines = [line.split() for line in memory.split('\n')]
        total = int(lines[0][1])

        if 'available' in lines[2][1]:
            available = int(lines[2][1])
        else:
            free = int(lines[1][1])
            cached = int(lines[3][1])
            available = free + cached

        used = total - available

        return {'used': used, 'total': total}
    except:
        return {}


def get_disks():
    try:
        disks = [line.split() for line in run(['df', '-P']).split('\n')[1:]]
        disks = [disk for disk in disks
                 if disk and disk[0].startswith('/dev/sd')]
        return [{'device': disk[0],
                 'size': int(disk[1]),
                 'used': int(disk[2]),
                 'available': int(disk[3]),
                 'mount': disk[5]} for disk in disks]
    except:
        return []


def get_traffic():
    try:
        lines = [line.split() for line in
                 run(['vnstat', '-tr']).split('\n')[3:5]]
        # run(['vnstat', '-tr', '-i', 'wlp0s20u3u1']).split('\n')[3:5]]
        rx = float(lines[0][1])
        tx = float(lines[1][1])
        return {'rx': rx,
                'tx': tx}
    except:
        return {}


if __name__ == '__main__':
    main()
