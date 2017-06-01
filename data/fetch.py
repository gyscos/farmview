#!/usr/bin/env python3

import json
import subprocess
import argparse


def run(args, **kwargs):
    return subprocess.check_output(args, universal_newlines=True, **kwargs)


def main():
    # Expects one argument: the network interface name
    parser = argparse.ArgumentParser(description='fetches various information')
    parser.add_argument('iface', metavar='IFACE',
                        help='network interface to monitor')
    args = parser.parse_args()

    print(json.dumps({
        'hostname': get_hostname(),
        'nproc': get_nproc(),
        'uptime': get_uptime(),
        'memory': get_memory_info(),
        'disks': get_disks(),
        'network': get_network(args.iface),
        'power': get_power()
    }))


def get_hostname():
    return run('hostname').strip()


def get_nproc():
    try:
        return int(run('nproc'))
    except:
        return None

def get_power():
    try:
        line = run(['sudo', 'ipmitool', 'sensor', 'reading', 'Current 1'])
        current = float(line.split('|')[1].strip())

        return { 'current': current }
    except:
        return None


def get_uptime():
    try:
        uptime = run('uptime').split(':')[-1]
        tokens = [float(token.strip()) for token in uptime.split(',')]
        return tokens
    except:
        return None


def get_memory_info():
    try:
        memory = run(['head', '-n', '4', '/proc/meminfo'])
        lines = [line.split() for line in memory.split('\n')]
        total = int(lines[0][1])

        if 'Available' in lines[2][0]:
            available = int(lines[2][1])
        else:
            free = int(lines[1][1])
            cached = int(lines[3][1])
            available = free + cached

        used = total - available

        return {'used': used * 1024, 'total': total * 1024}
    except:
        return None


def get_disks():
    try:
        disks = [line.split() for line in run(['df', '-P']).split('\n')[1:]]
        disks = [disk for disk in disks
                 if disk and not disk[0] in ["tmpfs", "udev", "cgmfs", "none"]]

        def get_model(device):
            fallback = None

            try:
                lines = run(['sudo', 'smartctl', '-i', device]).split('\n')

                def to_spec(line):
                    colon = line.index(':')
                    return (line[:colon], line[colon+1:].strip())

                specs = [to_spec(line) for line in lines if ':' in line]
                specs = {spec[0]: spec[1] for spec in specs}

                if 'Device Model' in specs:
                    return specs['Device Model']

                return specs['Vendor'] + ' ' + specs['Product']
            except:
                return None



        return sorted([{'device': disk[0],
                 'model': get_model(disk[0]),
                 'size': int(disk[1]) * 1024,
                 'used': int(disk[2]) * 1024,
                 'available': int(disk[3]) * 1024,
                 'mount': disk[5]} for disk in disks],
                 key = lambda disk: disk['mount'])
    except:
        return []


def get_network(iface):
    result = {}

    try:
        network_ip = run(['ip', 'addr', 'show', iface, 'scope', 'global']).split('\n')[2].split()[1]
        result['ip'] = network_ip.split('/')[0]
    except:
        pass

    try:
        tokens = run(['ifstat', '-i', iface, '5', '1']).split('\n')[2].split()
        result['rx'] = int(float(tokens[0]) * 1024)
        result['tx'] = int(float(tokens[1]) * 1024)
    except:
        pass

    return result


if __name__ == '__main__':
    main()
