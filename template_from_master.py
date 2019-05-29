from scg.config_parser import parse_config

def main():
    cfg = parse_config('johan_sverdrup.yaml').data

if '__main__' == __name__:
    main()