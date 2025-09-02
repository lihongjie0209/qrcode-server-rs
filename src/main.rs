use axum::{
    extract::{Multipart, Query, WebSocketUpgrade, ws::{WebSocket, Message}},
    http::StatusCode,
    response::{Html, Json, Response, Redirect},
    routing::{get, post},
    Router,
};
use opencv::{
    core::{Mat, Vector},
    imgcodecs::{IMREAD_COLOR},
    wechat_qrcode::WeChatQRCode,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, time::Instant};
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error, debug};
use object_pool::Pool;
use futures_util::{SinkExt, StreamExt};
use base64::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
struct QRCodeResult {
    text: String,
    points: Vec<[f32; 2]>,
    bbox: BoundingBox,
}

#[derive(Debug, Serialize, Deserialize)]
struct BoundingBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Serialize)]
struct DetectionStatistics {
    image_decode_time_ms: f64,
    detection_time_ms: f64,
    total_time_ms: f64,
    image_width: i32,
    image_height: i32,
    pool_acquisition_time_ms: f64,
}

#[derive(Debug, Serialize)]
struct QRCodeInfo {
    text: String,
    position: Position,
}

#[derive(Debug, Serialize)]
struct Position {
    coordinates: Vec<(f32, f32)>,
}

#[derive(Debug, Serialize)]
struct DetectionResponse {
    success: bool,
    message: String,
    qrcodes: Vec<QRCodeResult>,
    count: usize,
    statistics: DetectionStatistics,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebSocketRequest {
    #[serde(rename = "type")]
    msg_type: String,
    image: Option<String>, // Base64 编码的图片数据
}

#[derive(Debug, Serialize)]
struct WebSocketResponse {
    #[serde(rename = "type")]
    msg_type: String,
    success: bool,
    message: String,
    qrcodes: Option<Vec<QRCodeResult>>,
    count: Option<usize>,
    statistics: Option<DetectionStatistics>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HealthQuery {
    #[serde(default)]
    verbose: bool,
}

struct QRCodeDetector {
    detector: WeChatQRCode,
}

impl QRCodeDetector {
    fn new() -> opencv::Result<Self> {
        // WeChat QRCode模型文件路径
        let detector_prototxt = "models/detect.prototxt";
        let detector_caffemodel = "models/detect.caffemodel";
        let super_resolution_prototxt = "models/sr.prototxt";
        let super_resolution_caffemodel = "models/sr.caffemodel";
        
        let detector = WeChatQRCode::new(
            detector_prototxt,
            detector_caffemodel,
            super_resolution_prototxt,
            super_resolution_caffemodel,
        )?;

        Ok(Self { detector })
    }

    fn detect_qr_codes(&mut self, image: &Mat) -> Result<Vec<QRCodeResult>, Box<dyn std::error::Error>> {
        let mut points = Vector::<Mat>::new();
        
        let decoded_info = self.detector.detect_and_decode(image, &mut points)?;
        
        if decoded_info.is_empty() {
            return Ok(vec![]);
        }
        
        let mut results = Vec::new();
        
        for i in 0..decoded_info.len() {
            let text = decoded_info.get(i)?;
            
            // 提取四个角点坐标
            if i < points.len() {
                let point_mat = points.get(i)?;
                
                // WeChat QRCode返回的points是每个QR码的四个角点
                // point_mat 是一个 8x1 的矩阵，包含 4 个点的 x,y 坐标
                // 格式: [x1, y1, x2, y2, x3, y3, x4, y4]
                let mut coordinates = Vec::new();
                
                if point_mat.rows() >= 4 && point_mat.cols() >= 2 {
                    // 读取四个角点
                    for j in 0..4 {
                        let x = (*point_mat.at_2d::<f32>(j, 0)? * 10.0).round() / 10.0;  // 保留1位小数
                        let y = (*point_mat.at_2d::<f32>(j, 1)? * 10.0).round() / 10.0;  // 保留1位小数
                        coordinates.push([x, y]);
                    }
                } else if point_mat.total() >= 8 {
                    // 如果是8x1矩阵，按顺序读取
                    for j in 0..4 {
                        let x = (*point_mat.at::<f32>(j * 2)? * 10.0).round() / 10.0;      // 保留1位小数
                        let y = (*point_mat.at::<f32>(j * 2 + 1)? * 10.0).round() / 10.0;  // 保留1位小数
                        coordinates.push([x, y]);
                    }
                } else {
                    // 如果角点数据无效，创建一个基于图片尺寸的默认区域
                    warn!("Invalid corner points data for QR code {}, using fallback", i);
                    let img_width = image.cols() as f32;
                    let img_height = image.rows() as f32;
                    
                    let size = (img_width.min(img_height) * 0.6) as f32;
                    let x_offset = (img_width - size) / 2.0;
                    let y_offset = (img_height - size) / 2.0;
                    
                    coordinates = vec![
                        [x_offset, y_offset],
                        [x_offset + size, y_offset],
                        [x_offset + size, y_offset + size],
                        [x_offset, y_offset + size],
                    ];
                }
                
                // 计算边界框
                if !coordinates.is_empty() {
                    let min_x = coordinates.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
                    let max_x = coordinates.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max);
                    let min_y = coordinates.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
                    let max_y = coordinates.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
                    
                    // 边界框坐标也保留1位小数
                    let bbox_x = (min_x * 10.0).round() / 10.0;
                    let bbox_y = (min_y * 10.0).round() / 10.0;
                    let bbox_width = ((max_x - min_x) * 10.0).round() / 10.0;
                    let bbox_height = ((max_y - min_y) * 10.0).round() / 10.0;
                    
                    results.push(QRCodeResult {
                        text,
                        points: coordinates,
                        bbox: BoundingBox {
                            x: bbox_x,
                            y: bbox_y,
                            width: bbox_width,
                            height: bbox_height,
                        },
                    });
                }
            } else {
                // 如果没有角点数据，使用备用方案
                warn!("No corner points data for QR code {}, using fallback", i);
                let img_width = image.cols() as f32;
                let img_height = image.rows() as f32;
                
                let size = (img_width.min(img_height) * 0.6) as f32;
                let x_offset = (img_width - size) / 2.0;
                let y_offset = (img_height - size) / 2.0;
                
                let coordinates = vec![
                    [x_offset, y_offset],
                    [x_offset + size, y_offset],
                    [x_offset + size, y_offset + size],
                    [x_offset, y_offset + size],
                ];
                
                let min_x = coordinates.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
                let max_x = coordinates.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max);
                let min_y = coordinates.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
                let max_y = coordinates.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
                
                results.push(QRCodeResult {
                    text,
                    points: coordinates,
                    bbox: BoundingBox {
                        x: min_x,
                        y: min_y,
                        width: max_x - min_x,
                        height: max_y - min_y,
                    },
                });
            }
        }
        
        Ok(results)
    }
}

// 全局对象池
type DetectorPool = Pool<QRCodeDetector>;
static mut POOL: Option<DetectorPool> = None;
static POOL_INIT: std::sync::Once = std::sync::Once::new();

// 对象池配置
fn get_pool_config() -> (usize, usize) {
    let initial_size = std::env::var("POOL_INITIAL_SIZE")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<usize>()
        .unwrap_or(10)
        .max(1)
        .min(100);
    
    let max_size = std::env::var("POOL_MAX_SIZE")
        .unwrap_or_else(|_| "50".to_string())
        .parse::<usize>()
        .unwrap_or(50)
        .max(initial_size)
        .min(200);
    
    (initial_size, max_size)
}

fn get_detector_pool() -> &'static DetectorPool {
    unsafe {
        POOL_INIT.call_once(|| {
            let (initial_size, _max_size) = get_pool_config();
            info!("Initializing QRCode detector pool with {} initial instances...", initial_size);
            
            let pool = Pool::new(initial_size, || {
                match QRCodeDetector::new() {
                    Ok(detector) => detector,
                    Err(e) => {
                        error!("Failed to create QRCode detector: {}", e);
                        panic!("Cannot create QRCode detector");
                    }
                }
            });
            
            POOL = Some(pool);
            info!("QRCode detector pool initialized successfully");
        });
        POOL.as_ref().unwrap()
    }
}

// 检查模型文件是否存在的函数
fn check_model_files() -> Result<(), String> {
    let model_files = [
        "models/detect.prototxt",
        "models/detect.caffemodel", 
        "models/sr.prototxt",
        "models/sr.caffemodel"
    ];
    
    for model_file in &model_files {
        if !PathBuf::from(model_file).exists() {
            return Err(format!("WeChat QRCode model file not found: {}", model_file));
        }
    }
    
    info!("All WeChat QRCode model files found");
    Ok(())
}

async fn health_check(Query(params): Query<HealthQuery>) -> Json<serde_json::Value> {
    let _pool = get_detector_pool();
    let (initial_size, max_size) = get_pool_config();
    
    let mut response = serde_json::json!({
        "status": "healthy",
        "service": "qrcode-detector",
        "version": "0.1.0",
        "pool_stats": {
            "max_size": max_size,
            "initial_size": initial_size
        }
    });

    if params.verbose {
        response["opencv_version"] = serde_json::json!("4.6.0");
        response["features"] = serde_json::json!({
            "wechat_qrcode": true,
            "file_upload": true,
            "base64_input": true,
            "object_pool": true
        });
    }

    Json(response)
}

async fn serve_frontend() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn serve_camera_scanner() -> Html<&'static str> {
    Html(include_str!("../static/camera_qr_scanner.html"))
}

async fn serve_static_files() -> Result<Response, StatusCode> {
    // 这里可以扩展为完整的静态文件服务
    // 目前只返回404，因为我们的前端是单页面应用
    Err(StatusCode::NOT_FOUND)
}

async fn detect_from_file(mut multipart: Multipart) -> Result<Json<DetectionResponse>, StatusCode> {
    let start_time = Instant::now();
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("");
        
        if name == "image" || name == "file" {
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            
            // 图像解码时间统计
            let decode_start = Instant::now();
            
            // 将字节数据转换为OpenCV Mat
            let image_vec = data.to_vec();
            let mat = Mat::from_slice(&image_vec).map_err(|e| {
                error!("Failed to create Mat from bytes: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            
            let image = opencv::imgcodecs::imdecode(&mat, IMREAD_COLOR).map_err(|e| {
                error!("Failed to decode image: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            
            let decode_time = decode_start.elapsed().as_secs_f64() * 1000.0;
            
            if image.empty() {
                warn!("Received empty or invalid image");
                return Ok(Json(DetectionResponse {
                    success: false,
                    message: "Invalid image format".to_string(),
                    qrcodes: Vec::new(),
                    count: 0,
                    statistics: DetectionStatistics {
                        image_decode_time_ms: decode_time,
                        detection_time_ms: 0.0,
                        total_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                        image_width: 0,
                        image_height: 0,
                        pool_acquisition_time_ms: 0.0,
                    },
                }));
            }
            
            let image_size = image.size().unwrap();
            info!("Image decoded successfully ({}x{}), detecting QRCodes...", image_size.width, image_size.height);
            
            // 从对象池获取检测器
            let pool_start = Instant::now();
            let pool = get_detector_pool();
            let mut detector = pool.pull(|| {
                match QRCodeDetector::new() {
                    Ok(detector) => detector,
                    Err(e) => {
                        error!("Failed to create fallback QRCode detector: {}", e);
                        panic!("Cannot create fallback QRCode detector");
                    }
                }
            });
            let pool_acquisition_time = pool_start.elapsed().as_secs_f64() * 1000.0;
            
            // 二维码检测时间统计
            let detection_start = Instant::now();
            let qrcodes = detector.detect_qr_codes(&image).map_err(|e| {
                error!("QRCode detection failed: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            let detection_time = detection_start.elapsed().as_secs_f64() * 1000.0;
            
            // 检测器会自动归还到池中（通过Drop trait）
            
            let total_time = start_time.elapsed().as_secs_f64() * 1000.0;
            
            info!("Detected {} QR codes in uploaded image (decode: {:.2}ms, pool: {:.2}ms, detection: {:.2}ms, total: {:.2}ms)", 
                  qrcodes.len(), decode_time, pool_acquisition_time, detection_time, total_time);
            
            return Ok(Json(DetectionResponse {
                success: true,
                message: format!("Detected {} QR code(s)", qrcodes.len()),
                count: qrcodes.len(),
                qrcodes,
                statistics: DetectionStatistics {
                    image_decode_time_ms: decode_time,
                    detection_time_ms: detection_time,
                    total_time_ms: total_time,
                    image_width: image_size.width,
                    image_height: image_size.height,
                    pool_acquisition_time_ms: pool_acquisition_time,
                },
            }));
        }
    }
    
    Err(StatusCode::BAD_REQUEST)
}

async fn detect_from_base64(
    Json(payload): Json<HashMap<String, String>>,
) -> Result<Json<DetectionResponse>, StatusCode> {
    let start_time = Instant::now();
    let base64_data = payload.get("image").ok_or(StatusCode::BAD_REQUEST)?;
    
    // 图像解码时间统计
    let decode_start = Instant::now();
    
    // 解码base64数据
    use base64::prelude::*;
    let image_data = BASE64_STANDARD.decode(base64_data).map_err(|e| {
        error!("Failed to decode base64: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    
    // 转换为OpenCV Mat
    let mat = Mat::from_slice(&image_data).map_err(|e| {
        error!("Failed to create Mat from base64 data: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    
    let image = opencv::imgcodecs::imdecode(&mat, IMREAD_COLOR).map_err(|e| {
        error!("Failed to decode image from base64: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    
    let decode_time = decode_start.elapsed().as_secs_f64() * 1000.0;
    
    if image.empty() {
        warn!("Received empty or invalid image from base64");
        return Ok(Json(DetectionResponse {
            success: false,
            message: "Invalid image format".to_string(),
            qrcodes: Vec::new(),
            count: 0,
            statistics: DetectionStatistics {
                image_decode_time_ms: decode_time,
                detection_time_ms: 0.0,
                total_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                image_width: 0,
                image_height: 0,
                pool_acquisition_time_ms: 0.0,
            },
        }));
    }
    
    let image_size = image.size().unwrap();
    info!("Base64 image decoded successfully ({}x{}), detecting QRCodes...", image_size.width, image_size.height);
    
    // 从对象池获取检测器
    let pool_start = Instant::now();
    let pool = get_detector_pool();
    let mut detector = pool.pull(|| {
        match QRCodeDetector::new() {
            Ok(detector) => detector,
            Err(e) => {
                error!("Failed to create fallback QRCode detector: {}", e);
                panic!("Cannot create fallback QRCode detector");
            }
        }
    });
    let pool_acquisition_time = pool_start.elapsed().as_secs_f64() * 1000.0;
    
    // 二维码检测时间统计
    let detection_start = Instant::now();
    let qrcodes = detector.detect_qr_codes(&image).map_err(|e| {
        error!("QRCode detection failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let detection_time = detection_start.elapsed().as_secs_f64() * 1000.0;
    
    let total_time = start_time.elapsed().as_secs_f64() * 1000.0;
    
    info!("Detected {} QR codes in base64 image (decode: {:.2}ms, pool: {:.2}ms, detection: {:.2}ms, total: {:.2}ms)", 
          qrcodes.len(), decode_time, pool_acquisition_time, detection_time, total_time);
    
    Ok(Json(DetectionResponse {
        success: true,
        message: format!("Detected {} QR code(s)", qrcodes.len()),
        count: qrcodes.len(),
        qrcodes,
        statistics: DetectionStatistics {
            image_decode_time_ms: decode_time,
            detection_time_ms: detection_time,
            total_time_ms: total_time,
            image_width: image_size.width,
            image_height: image_size.height,
            pool_acquisition_time_ms: pool_acquisition_time,
        },
    }))
}

// WebSocket 升级处理函数
async fn websocket_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_websocket)
}

// WebSocket 连接处理
async fn handle_websocket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    
    info!("New WebSocket connection established");
    
    // 发送连接确认消息
    let welcome_msg = WebSocketResponse {
        msg_type: "connected".to_string(),
        success: true,
        message: "WebSocket connected successfully".to_string(),
        qrcodes: None,
        count: None,
        statistics: None,
        error: None,
    };
    
    if let Ok(welcome_json) = serde_json::to_string(&welcome_msg) {
        if sender.send(Message::Text(welcome_json)).await.is_err() {
            error!("Failed to send welcome message");
            return;
        }
    }
    
    // 处理接收到的消息
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("Received WebSocket text message: {}", text);
                
                // 解析请求
                match serde_json::from_str::<WebSocketRequest>(&text) {
                    Ok(request) => {
                        let response = handle_websocket_request(request).await;
                        
                        if let Ok(response_json) = serde_json::to_string(&response) {
                            if sender.send(Message::Text(response_json)).await.is_err() {
                                error!("Failed to send WebSocket response");
                                break;
                            }
                        }
                        
                        // 如果是关闭请求，结束连接
                        if response.msg_type == "close" {
                            info!("WebSocket connection closed by client request");
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse WebSocket request: {}", e);
                        let error_response = WebSocketResponse {
                            msg_type: "error".to_string(),
                            success: false,
                            message: "Invalid request format".to_string(),
                            qrcodes: None,
                            count: None,
                            statistics: None,
                            error: Some(format!("Parse error: {}", e)),
                        };
                        
                        if let Ok(error_json) = serde_json::to_string(&error_response) {
                            if sender.send(Message::Text(error_json)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
            Ok(Message::Binary(data)) => {
                debug!("Received WebSocket binary message: {} bytes", data.len());
                
                // 处理二进制图片数据
                let response = handle_websocket_binary_request(data).await;
                
                if let Ok(response_json) = serde_json::to_string(&response) {
                    if sender.send(Message::Text(response_json)).await.is_err() {
                        error!("Failed to send WebSocket binary response");
                        break;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed by client");
                break;
            }
            Ok(Message::Ping(data)) => {
                debug!("Received WebSocket ping");
                if sender.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                debug!("Received WebSocket pong");
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }
    
    info!("WebSocket connection terminated");
}

// 处理WebSocket文本请求
async fn handle_websocket_request(request: WebSocketRequest) -> WebSocketResponse {
    match request.msg_type.as_str() {
        "detect" => {
            if let Some(image_data) = request.image {
                // 使用现有的base64检测逻辑
                match detect_qr_from_base64(image_data).await {
                    Ok(detection_result) => WebSocketResponse {
                        msg_type: "detection_result".to_string(),
                        success: detection_result.success,
                        message: detection_result.message,
                        qrcodes: Some(detection_result.qrcodes),
                        count: Some(detection_result.count),
                        statistics: Some(detection_result.statistics),
                        error: None,
                    },
                    Err(e) => WebSocketResponse {
                        msg_type: "error".to_string(),
                        success: false,
                        message: "Detection failed".to_string(),
                        qrcodes: None,
                        count: None,
                        statistics: None,
                        error: Some(e),
                    }
                }
            } else {
                WebSocketResponse {
                    msg_type: "error".to_string(),
                    success: false,
                    message: "Missing image data".to_string(),
                    qrcodes: None,
                    count: None,
                    statistics: None,
                    error: Some("No image field in request".to_string()),
                }
            }
        }
        "close" => {
            WebSocketResponse {
                msg_type: "close".to_string(),
                success: true,
                message: "Connection closing".to_string(),
                qrcodes: None,
                count: None,
                statistics: None,
                error: None,
            }
        }
        _ => {
            WebSocketResponse {
                msg_type: "error".to_string(),
                success: false,
                message: format!("Unknown message type: {}", request.msg_type),
                qrcodes: None,
                count: None,
                statistics: None,
                error: Some("Unsupported message type".to_string()),
            }
        }
    }
}

// 处理WebSocket二进制请求
async fn handle_websocket_binary_request(binary_data: Vec<u8>) -> WebSocketResponse {
    match detect_qr_from_binary(binary_data).await {
        Ok(detection_result) => WebSocketResponse {
            msg_type: "detection_result".to_string(),
            success: detection_result.success,
            message: detection_result.message,
            qrcodes: Some(detection_result.qrcodes),
            count: Some(detection_result.count),
            statistics: Some(detection_result.statistics),
            error: None,
        },
        Err(e) => WebSocketResponse {
            msg_type: "error".to_string(),
            success: false,
            message: "Binary detection failed".to_string(),
            qrcodes: None,
            count: None,
            statistics: None,
            error: Some(e),
        }
    }
}

// 从base64数据检测QR码的辅助函数
async fn detect_qr_from_base64(base64_data: String) -> Result<DetectionResponse, String> {
    let start_time = Instant::now();
    
    // 图像解码时间统计
    let decode_start = Instant::now();
    
    // 解码base64数据
    let image_data = BASE64_STANDARD.decode(&base64_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    
    // 转换为OpenCV Mat
    let mat = Mat::from_slice(&image_data)
        .map_err(|e| format!("Failed to create Mat from base64 data: {}", e))?;
    
    let image = opencv::imgcodecs::imdecode(&mat, IMREAD_COLOR)
        .map_err(|e| format!("Failed to decode image from base64: {}", e))?;
    
    let decode_time = decode_start.elapsed().as_secs_f64() * 1000.0;
    
    if image.empty() {
        return Ok(DetectionResponse {
            success: false,
            message: "Invalid image format".to_string(),
            qrcodes: Vec::new(),
            count: 0,
            statistics: DetectionStatistics {
                image_decode_time_ms: decode_time,
                detection_time_ms: 0.0,
                total_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                image_width: 0,
                image_height: 0,
                pool_acquisition_time_ms: 0.0,
            },
        });
    }
    
    let image_size = image.size().unwrap();
    
    // 从对象池获取检测器
    let pool_start = Instant::now();
    let pool = get_detector_pool();
    let mut detector = pool.pull(|| {
        match QRCodeDetector::new() {
            Ok(detector) => detector,
            Err(e) => {
                error!("Failed to create fallback QRCode detector: {}", e);
                panic!("Cannot create fallback QRCode detector");
            }
        }
    });
    let pool_acquisition_time = pool_start.elapsed().as_secs_f64() * 1000.0;
    
    // 二维码检测时间统计
    let detection_start = Instant::now();
    let qrcodes = detector.detect_qr_codes(&image)
        .map_err(|e| format!("QRCode detection failed: {}", e))?;
    let detection_time = detection_start.elapsed().as_secs_f64() * 1000.0;
    
    let total_time = start_time.elapsed().as_secs_f64() * 1000.0;
    
    Ok(DetectionResponse {
        success: true,
        message: format!("Detected {} QR code(s)", qrcodes.len()),
        count: qrcodes.len(),
        qrcodes,
        statistics: DetectionStatistics {
            image_decode_time_ms: decode_time,
            detection_time_ms: detection_time,
            total_time_ms: total_time,
            image_width: image_size.width,
            image_height: image_size.height,
            pool_acquisition_time_ms: pool_acquisition_time,
        },
    })
}

// 从二进制数据检测QR码的辅助函数
async fn detect_qr_from_binary(binary_data: Vec<u8>) -> Result<DetectionResponse, String> {
    let start_time = Instant::now();
    
    // 图像解码时间统计
    let decode_start = Instant::now();
    
    // 转换为OpenCV Mat
    let mat = Mat::from_slice(&binary_data)
        .map_err(|e| format!("Failed to create Mat from binary data: {}", e))?;
    
    let image = opencv::imgcodecs::imdecode(&mat, IMREAD_COLOR)
        .map_err(|e| format!("Failed to decode image from binary data: {}", e))?;
    
    let decode_time = decode_start.elapsed().as_secs_f64() * 1000.0;
    
    if image.empty() {
        return Ok(DetectionResponse {
            success: false,
            message: "Invalid image format".to_string(),
            qrcodes: Vec::new(),
            count: 0,
            statistics: DetectionStatistics {
                image_decode_time_ms: decode_time,
                detection_time_ms: 0.0,
                total_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                image_width: 0,
                image_height: 0,
                pool_acquisition_time_ms: 0.0,
            },
        });
    }
    
    let image_size = image.size().unwrap();
    
    // 从对象池获取检测器
    let pool_start = Instant::now();
    let pool = get_detector_pool();
    let mut detector = pool.pull(|| {
        match QRCodeDetector::new() {
            Ok(detector) => detector,
            Err(e) => {
                error!("Failed to create fallback QRCode detector: {}", e);
                panic!("Cannot create fallback QRCode detector");
            }
        }
    });
    let pool_acquisition_time = pool_start.elapsed().as_secs_f64() * 1000.0;
    
    // 二维码检测时间统计
    let detection_start = Instant::now();
    let qrcodes = detector.detect_qr_codes(&image)
        .map_err(|e| format!("QRCode detection failed: {}", e))?;
    let detection_time = detection_start.elapsed().as_secs_f64() * 1000.0;
    
    let total_time = start_time.elapsed().as_secs_f64() * 1000.0;
    
    Ok(DetectionResponse {
        success: true,
        message: format!("Detected {} QR code(s)", qrcodes.len()),
        count: qrcodes.len(),
        qrcodes,
        statistics: DetectionStatistics {
            image_decode_time_ms: decode_time,
            detection_time_ms: detection_time,
            total_time_ms: total_time,
            image_width: image_size.width,
            image_height: image_size.height,
            pool_acquisition_time_ms: pool_acquisition_time,
        },
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("Starting QRCode detection server...");
    
    // 获取端口配置
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    
    // 获取对象池配置
    let (initial_pool_size, max_pool_size) = get_pool_config();
    
    // 获取上下文路径配置
    let context_path = std::env::var("CONTEXT_PATH").unwrap_or_else(|_| "/".to_string());
    let context_path = if context_path.starts_with('/') {
        context_path
    } else {
        format!("/{}", context_path)
    };
    let context_path = if context_path.ends_with('/') && context_path.len() > 1 {
        context_path.trim_end_matches('/').to_string()
    } else {
        context_path
    };
    
    // 显示配置信息
    info!("Server configuration:");
    info!("  Port: {}", port);
    info!("  Context path: {}", context_path);
    info!("  Pool initial size: {}", initial_pool_size);
    info!("  Pool max size: {}", max_pool_size);
    
    // 检查WeChat QRCode模型文件是否存在
    check_model_files().map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    
    // 初始化对象池
    info!("Initializing detector pool...");
    let _pool = get_detector_pool();
    info!("Detector pool initialized with {} initial instances (max: {})", initial_pool_size, max_pool_size);
    
    // 构建路由
    let app = if context_path == "/" {
        // 默认根路径
        Router::new()
            .route("/", get(serve_frontend))
            .route("/camera", get(serve_camera_scanner))
            .route("/ws", get(websocket_handler))
            .route("/health", get(health_check))
            .route("/detect/file", post(detect_from_file))
            .route("/detect/base64", post(detect_from_base64))
    } else {
        // 自定义上下文路径
        let context_clone = context_path.clone();
        Router::new()
            .route(&format!("{}/", context_path), get(serve_frontend))
            .route(&format!("{}/camera", context_path), get(serve_camera_scanner))
            .route(&format!("{}/ws", context_path), get(websocket_handler))
            .route(&format!("{}/health", context_path), get(health_check))
            .route(&format!("{}/detect/file", context_path), post(detect_from_file))
            .route(&format!("{}/detect/base64", context_path), post(detect_from_base64))
            // 添加根路径重定向到上下文路径
            .route("/", get(move || async move {
                Redirect::permanent(&format!("{}/", context_clone))
            }))
    };
    
    let app = app.layer(CorsLayer::permissive());
    
    let addr = format!("0.0.0.0:{}", port);
    
    info!("Server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
