use std::fs;
use std::process;

use tabled::settings::Height;
use tabled::settings::object::Segment;
use tabled::{
    settings::{Settings, Width, Style},
    Table, Tabled
};
use clap::Parser;
use terminal_size::{terminal_size, Height as TerminalHeight, Width as TerminalWidth};
use natord::compare;
use std::sync::mpsc;
use std::thread;
use serde_json;

use gethostname::gethostname;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Display errors only
    #[arg(short, long, default_value_t = false)]
    errors: bool,
    // Output format 
    // (table, csv, json)
    //#[arg(short, long, default_value = "table")]
    #[arg(short, long, value_parser, verbatim_doc_comment)]
    output_format: String,
}

fn get_terminal_size() -> (usize, usize) {
    let (TerminalWidth(width), TerminalHeight(height)) =
        terminal_size().expect("failed to obtain a terminal size");

    (width as usize, height as usize)
}

fn parse_result(device : Mlx5Port, _data: json::JsonValue, host_serial: String, host_name: String) -> Mlx5PortStats {

    let mut _mlx5_port_stats = Mlx5PortStats {
        raw_physical_errors_per_lane_0: _data["Physical Counters and BER Info"]["Raw Physical Errors Per Lane"]["values"][0].to_string().parse::<u64>().unwrap(),
        raw_physical_errors_per_lane_1: _data["Physical Counters and BER Info"]["Raw Physical Errors Per Lane"]["values"][1].to_string().parse::<u64>().unwrap(),
        raw_physical_errors_per_lane_2: _data["Physical Counters and BER Info"]["Raw Physical Errors Per Lane"]["values"][2].to_string().parse::<u64>().unwrap(),
        raw_physical_errors_per_lane_3: _data["Physical Counters and BER Info"]["Raw Physical Errors Per Lane"]["values"][3].to_string().parse::<u64>().unwrap(),
        effective_physical_errors: _data["Physical Counters and BER Info"]["Effective Physical Errors"].to_string().parse::<u64>().unwrap(),
        effective_physical_ber: _data["Physical Counters and BER Info"]["Effective Physical BER"].as_str().unwrap().to_string(),
        raw_physical_ber: _data["Physical Counters and BER Info"]["Raw Physical BER"].as_str().unwrap().to_string(),
        vendor_serial: _data["Module Info"]["Vendor Serial Number"].as_str().unwrap().to_string(),
        recommedation: _data["Troubleshooting Info"]["Recommendation"].as_str().unwrap().to_string(),
        link_state: _data["Operational Info"]["State"].as_str().unwrap().to_string(),
        fec_bin_0: _data["Histogram of FEC Errors"]["Bin 0"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_1: _data["Histogram of FEC Errors"]["Bin 1"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_2: _data["Histogram of FEC Errors"]["Bin 2"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_3: _data["Histogram of FEC Errors"]["Bin 3"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_4: _data["Histogram of FEC Errors"]["Bin 4"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_5: _data["Histogram of FEC Errors"]["Bin 5"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_6: _data["Histogram of FEC Errors"]["Bin 6"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_7: _data["Histogram of FEC Errors"]["Bin 7"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_8: _data["Histogram of FEC Errors"]["Bin 8"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_9: _data["Histogram of FEC Errors"]["Bin 9"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_10: _data["Histogram of FEC Errors"]["Bin 10"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_11: _data["Histogram of FEC Errors"]["Bin 11"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_12: _data["Histogram of FEC Errors"]["Bin 12"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_13: _data["Histogram of FEC Errors"]["Bin 13"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_14: _data["Histogram of FEC Errors"]["Bin 14"]["values"][1].to_string().parse::<u64>().unwrap(),
        fec_bin_15: _data["Histogram of FEC Errors"]["Bin 15"]["values"][1].to_string().parse::<u64>().unwrap(),
        device: device.device,
        port: device.port,
        comments: "".to_string(),
        host_serial,
        hostname: host_name,
        error: false,
};

    if _mlx5_port_stats.link_state == "Active" {
        _mlx5_port_stats.link_state = "✅ Active".to_string();
    } else {
        _mlx5_port_stats.link_state = format!("❌ {}",_mlx5_port_stats.link_state).to_string();
        _mlx5_port_stats.comments = "Link is down".to_string();
    }

    if _mlx5_port_stats.raw_physical_ber.parse::<f64>().unwrap() > 0.0000000010 {
        _mlx5_port_stats.comments = "❌ Physical BER is high".to_string();
    }

    if _mlx5_port_stats.effective_physical_errors > 0 {
        _mlx5_port_stats.comments = "❌ Physical errors detected".to_string();
    }

    if _mlx5_port_stats.fec_bin_7 > 0 {
        _mlx5_port_stats.comments = "❌ FEC[7] errors detected".to_string();
    }

    if _mlx5_port_stats.fec_bin_8 > 0 {
        _mlx5_port_stats.comments = "❌ FEC[8] errors detected".to_string();
    }

        if _mlx5_port_stats.fec_bin_9 > 0 {
        _mlx5_port_stats.comments = "❌ FEC[9] errors detected".to_string();
    }

    return _mlx5_port_stats;
}


#[derive(Tabled, Clone, serde::Serialize)]
struct Mlx5Port {
    device: String,
    port: String,
}

#[derive(Tabled, Clone, serde::Serialize)]
struct Mlx5PortStats {
    host_serial: String,
    hostname: String,
    #[tabled(inline)]
    port: String, 
    device: String,
    #[tabled(rename = "bin0")]
    fec_bin_0: u64,
    #[tabled(rename = "bin1")]
    fec_bin_1: u64,
    #[tabled(skip)]
    #[tabled(rename = "bin2")]
    fec_bin_2: u64,
    #[tabled(rename = "bin3")]
    #[tabled(skip)]
    fec_bin_3: u64,
    #[tabled(rename = "bin4")]
    #[tabled(skip)]
    fec_bin_4: u64,
    #[tabled(rename = "bin5")]
    #[tabled(skip)]
    fec_bin_5: u64,
    #[tabled(rename = "bin6")]
    fec_bin_6: u64,
    #[tabled(rename = "bin7")]
    fec_bin_7: u64,
    #[tabled(rename = "bin8")]
    fec_bin_8: u64,
    #[tabled(rename = "bin8")]
    fec_bin_9: u64,
    #[tabled(rename = "bin9")]
    fec_bin_10: u64,
    #[tabled(rename = "bin10")]
    fec_bin_11: u64,
    #[tabled(rename = "bin11")]
    fec_bin_12: u64,
    #[tabled(rename = "bin12")]
    fec_bin_13: u64,
    #[tabled(rename = "bin13")]
    fec_bin_14: u64,
    #[tabled(rename = "bin14")]
    fec_bin_15: u64,
    #[tabled(rename = "bin15")]
    #[tabled(rename = "raw err 0 ")]
    raw_physical_errors_per_lane_0: u64,
    #[tabled(rename = "raw err 1 ")]
    raw_physical_errors_per_lane_1: u64,
    #[tabled(rename = "raw err 2 ")]
    raw_physical_errors_per_lane_2: u64,
    #[tabled(rename = "raw err 3 ")]
    raw_physical_errors_per_lane_3: u64,
    link_state: String,
    vendor_serial: String,
    #[tabled(rename = "phys err")]
    effective_physical_errors: u64,
    #[tabled(rename = "phys ber")]
    effective_physical_ber: String,
    #[tabled(rename = "raw ber")]
    raw_physical_ber: String,
    recommedation: String,
    comments: String, 
    #[tabled(skip)]
    error: bool,

}


fn collect_mlxlink_output(device : &Mlx5Port) -> json::JsonValue {
    let command = format!("mlxlink -m -e -c -d {} --rx_fec_histogram --show_histogram --json", device.port);
    let output = process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .expect("failed to execute process");

    let json_data = json::parse(String::from_utf8_lossy(&output.stdout).as_ref()).unwrap();
    

    let _data = json_data["result"]["output"].clone();
    //println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    //println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    return _data;
}

fn process_rows(all_port_stats: Vec<Mlx5PortStats>, args: Args) -> Vec<Mlx5PortStats>{
    let _error = args.errors;
    let mut all_port_stats = all_port_stats;
    all_port_stats.sort_by(|a, b| compare(&a.port, &b.port));

    let mut r = Vec::new();
    
    
    for _mlx5_port_stats in all_port_stats {
        if _mlx5_port_stats.error == _error {
            r.push(_mlx5_port_stats);
        } 
    }

    return r;
}

fn print_output(all_port_stats: Vec<Mlx5PortStats>, args: Args) {
    let _error = args.errors;
    
    
    if args.output_format == "table"{ 
        let (width, _) = get_terminal_size();
        let table_settings = Settings::default()
        .with(Width::truncate(width))
        .with(Style::modern_rounded());
        let r = process_rows(all_port_stats, args);
        if r.len() == 0 {
            //println!("No errors found.");
            std::process::exit(0);
        }
    
        let mut table = Table::new(&r);
        table.modify(Segment::all(), Height::limit(1 ));
        table.with(table_settings);
        println!("{}", table);
    
    }
    else if args.output_format == "csv" { 
        let r = process_rows(all_port_stats, args);
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        for _mlx5_port_stats in r {
            wtr.serialize(_mlx5_port_stats).unwrap();
        }
        wtr.flush().unwrap();
    
    }

    else if args.output_format == "json" {
        let r = process_rows(all_port_stats, args);
        let json_data = serde_json::to_string(&r);
        println!("{}", json_data.unwrap());
        
    }


}

fn get_mlx5_ports() -> Vec<Mlx5Port> {
    let _rdma_ports_list = fs::read_dir("/sys/class/net")
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|device| device.starts_with("rdma"))
        .map(|device| Mlx5Port {
            device: device.clone(),
            port: fs::read_dir(format!("/sys/class/net/{}/device/infiniband", device))
                .unwrap()
                .map(|entry| entry.unwrap().file_name().into_string().unwrap())
                .collect::<Vec<String>>()
                .join(","),
        })
        .collect::<Vec<Mlx5Port>>();

    return _rdma_ports_list;
}

fn check_root() {
    #[link(name = "c")]
    extern "C" {
        fn geteuid() -> u32;
        fn getegid() -> u32;
    }

    let _timeout = 60;

    let _euid = unsafe { geteuid() };
    let _egid = unsafe { getegid() };

    if _euid != 0 {
        println!("You must be root to run this program.");
        std::process::exit(1);
    }
}

fn main() {
    
    check_root();

    let args = Args::parse();

    let _rdma_ports_list = get_mlx5_ports();
    let rdma_ports_list = _rdma_ports_list.clone();
    let mut _chassis_serial = fs::read_to_string("/sys/class/dmi/id/chassis_serial").unwrap();
    _chassis_serial = _chassis_serial.trim().to_string();
    let mut _hostname = gethostname().to_string_lossy().to_string();

    let mut all_port_stats = Vec::<Mlx5PortStats>::new();

    let (tx, rx) = mpsc::channel();
    for _mlx5_device in _rdma_ports_list {
        let tx = tx.clone();
        let _chassis_serial = _chassis_serial.clone();

        let _hostname = _hostname.clone();
        thread::spawn(move || {
            let _output = collect_mlxlink_output(&_mlx5_device);
            let mut _mlx5_port_stats = parse_result(_mlx5_device, _output, _chassis_serial, _hostname.clone());
            tx.send(_mlx5_port_stats).unwrap();
        });
    }

    for _ in rdma_ports_list {
        let _mlx5_port_stats = rx.recv().unwrap();
        all_port_stats.push(_mlx5_port_stats);
    }

    print_output(all_port_stats, args);
}