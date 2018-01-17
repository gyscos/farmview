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
        'power': get_power(),
        'gpu': get_gpu(),
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

        return {'current': current}
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
        try:
            devices = json.loads(
                run(['lsblk', '--json', '-b', '-p']))['blockdevices']
        except:
            lines = run(
                ['lsblk', '-b', '-P', '-oNAME,MOUNTPOINT,MODEL,SIZE']).split('\n')
            lines = [line.split('"') for line in lines if line]

            devices = [{
                'name': '/dev/' + line[1],
                'mountpoint': line[3],
                'model': line[5],
                'size': int(line[7])} for line in lines]

        def get_mounts():
            mounts = [line.split()
                      for line in run(['df', '-P']).split('\n')[1:]]
            mounts = [mount for mount in mounts
                      if mount and not mount[0] in ["tmpfs", "udev", "cgmfs", "none"]]
            return {mount[0]: {
                    'size': int(mount[1]) * 1024,
                    'used': int(mount[2]) * 1024,
                    'available': int(mount[3]) * 1024} for mount in mounts}

        mounts = get_mounts()

        # For each device:
        # - If itself or any child is mounted, only select the mounted one
        # - If any child, pick the largest. Otherwise pick self
        # Return those as unmounted.
        def find_mounted(device):
            result = []

            if device['name'] in mounts:
                # This is it!
                result += [device]

            if 'children' in device:
                for child in device['children']:
                    result += find_mounted(child)

            return result

        def find_largest(device):
            max_size = int(device['size'])
            largest = device

            if 'children' in device:
                for child in device['children']:
                    candidate = find_largest(child)
                    size = int(candidate['size'])
                    if size > max_size:
                        max_size = size
                        largest = candidate

            return largest

        def select_devices(device):
            mounted = find_mounted(device)
            if mounted is not None:
                for mount in mounted:
                    mount.update(mounts[mount['name']])
                return mounted

            largest = find_largest(device)
            return [largest]

        def with_device(device):
            devices = select_devices(device)
            for child in devices:
                child['model'] = get_model(device['name'])
                child['attrs'] = get_attrs(device['name'])
            return devices

        def get_model(device):
            try:
                while device[-1].isdigit():
                    device = device[:-1]

                lines = run(['sudo', 'smartctl', '-i', device]).split('\n')

                def to_spec(line):
                    colon = line.index(':')
                    return (line[:colon], line[colon + 1:].strip())

                specs = [to_spec(line) for line in lines if ':' in line]
                specs = {spec[0]: spec[1] for spec in specs}

                if 'Device Model' in specs:
                    return specs['Device Model']

                return specs['Vendor'] + ' ' + specs['Product']
            except:
                return None

        def get_attrs(device):
            try:
                while device[-1].isdigit():
                    device = device[:-1]

                lines = run(['sudo', 'smartctl', '-A', device]).split('\n')

                def to_attr(line):
                    tokens = line.split()
                    return (tokens[1], {'value': tokens[3], 'raw': ' '.join(tokens[9:])})

                # print(lines)
                attrs = [to_attr(line) for line in lines[7:] if line]
                # print(attrs)
                attrs = {attr[0]: attr[1] for attr in attrs}
                return attrs
            except:
                return None

        return sorted([result for device in devices for result in with_device(device)], key=lambda device: device['mountpoint'])
    except:
        return []


def get_network(iface):
    result = {}

    try:
        network_ip = run(['ip', 'addr', 'show', iface, 'scope', 'global']).split('\n')[
            2].split()[1]
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


def get_gpu():
    try:
        gpus = json.loads(run(['gpustat', '--json']))['gpus']
        return gpus
    except:
        return []


if __name__ == '__main__':
    main()
