#!/bin/bash

# uKeep Docker 镜像推送脚本
# 将本地已构建的镜像推送到远程VPS

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env.deploy"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 镜像配置
IMAGE_NAME="ukeep"
IMAGE_TAG="latest"
CONTAINER_NAME="ukeep"

# VPS 配置默认值
VPS_USER=""
VPS_HOST=""
VPS_PORT="22"
VPS_DEPLOY_PATH=""
VPS_SSH_KEY_PATH=""

# SSH 选项（在 load_env 后构建）
SSH_OPTS=""

print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

build_ssh_opts() {
    SSH_OPTS="-o ConnectTimeout=10 -o PasswordAuthentication=no -o BatchMode=yes"
    if [[ -n "${VPS_SSH_KEY_PATH}" ]]; then
        if [[ ! -f "${VPS_SSH_KEY_PATH}" ]]; then
            print_error "SSH key not found: ${VPS_SSH_KEY_PATH}"
            exit 1
        fi
        SSH_OPTS="${SSH_OPTS} -i ${VPS_SSH_KEY_PATH}"
    fi
}

load_env() {
    if [[ ! -f "${ENV_FILE}" ]]; then
        print_error "Config file not found: ${ENV_FILE}"
        print_info "Create it from template: cp .env.deploy.example .env.deploy"
        exit 1
    fi
    source "${ENV_FILE}"

    if [[ -z "${VPS_HOST}" || -z "${VPS_USER}" ]]; then
        print_error "VPS_HOST and VPS_USER must be set in ${ENV_FILE}"
        exit 1
    fi
    build_ssh_opts
    print_info "Loaded config from ${ENV_FILE}"
}

check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker daemon is not running"
        exit 1
    fi
}

check_image() {
    if ! docker image inspect ${IMAGE_NAME}:${IMAGE_TAG} > /dev/null 2>&1; then
        print_error "Image ${IMAGE_NAME}:${IMAGE_TAG} not found. Run './docker-deploy.sh build' first."
        exit 1
    fi
    print_info "Found image: ${IMAGE_NAME}:${IMAGE_TAG}"
}

check_ssh() {
    print_info "Testing SSH connection to ${VPS_USER}@${VPS_HOST}..."
    if ! ssh -p ${VPS_PORT} ${SSH_OPTS} ${VPS_USER}@${VPS_HOST} "echo 'SSH OK'" > /dev/null 2>&1; then
        print_error "Cannot connect to VPS. Check SSH key and network."
        exit 1
    fi
    print_info "SSH connection OK"
}

push_image() {
    local image_file="/tmp/${IMAGE_NAME}-${IMAGE_TAG}.tar"
    local scp_opts="-o PasswordAuthentication=no -o BatchMode=yes"
    if [[ -n "${VPS_SSH_KEY_PATH}" ]]; then
        scp_opts="${scp_opts} -i ${VPS_SSH_KEY_PATH}"
    fi

    print_info "Saving image to archive..."
    docker save ${IMAGE_NAME}:${IMAGE_TAG} -o ${image_file}
    local size=$(du -h ${image_file} | cut -f1)
    print_info "Image saved: ${image_file} (${size})"

    print_info "Transferring image to VPS..."
    scp -P ${VPS_PORT} ${scp_opts} ${image_file} ${VPS_USER}@${VPS_HOST}:/tmp/

    print_info "Loading image on VPS..."
    ssh -p ${VPS_PORT} ${SSH_OPTS} ${VPS_USER}@${VPS_HOST} "docker load -i /tmp/${IMAGE_NAME}-${IMAGE_TAG}.tar && rm /tmp/${IMAGE_NAME}-${IMAGE_TAG}.tar"

    rm ${image_file}
    print_info "Image pushed successfully"
}

deploy_on_vps() {
    print_info "Deploying on VPS..."
    ssh -p ${VPS_PORT} ${SSH_OPTS} ${VPS_USER}@${VPS_HOST} << EOF
        cd ${VPS_DEPLOY_PATH}
        docker stop ${CONTAINER_NAME} 2>/dev/null || true
        docker rm ${CONTAINER_NAME} 2>/dev/null || true
        docker run -d --name ${CONTAINER_NAME} -p 80:80 --restart unless-stopped ${IMAGE_NAME}:${IMAGE_TAG}
        echo "Container started"
        docker ps | grep ${CONTAINER_NAME}
EOF
    print_info "Deployment completed"
}

show_help() {
    cat << EOF
uKeep Docker 镜像推送脚本

用法: $0 [命令]

命令:
    push        仅推送镜像到VPS (默认)
    deploy      推送镜像并部署
    help        显示帮助

配置文件: .env.deploy (从 .env.deploy.example 复制)

EOF
}

main() {
    local command=${1:-push}

    case $command in
        push)
            load_env
            check_docker
            check_image
            check_ssh
            push_image
            ;;
        deploy)
            load_env
            check_docker
            check_image
            check_ssh
            push_image
            deploy_on_vps
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "Unknown command: $command"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
