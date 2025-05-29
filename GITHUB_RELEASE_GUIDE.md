# GitHub发布指南

**Author:** Evan Liu <evan.liu@voltageenergy.com>  
**项目:** Voltage Modbus  
**版本:** 0.1.0

## 🚀 GitHub仓库发布步骤

### 1. 创建GitHub仓库

1. **登录GitHub**: 访问 https://github.com
2. **创建新仓库**:
   - 点击右上角的 "+" 按钮
   - 选择 "New repository"
   - 仓库名称: `voltage_modbus`
   - 描述: `High-performance Modbus TCP/RTU implementation for industrial automation and IoT applications`
   - 设置为 Public（公开）
   - **不要**初始化README、.gitignore或LICENSE（我们已经有了）

### 2. 推送代码到GitHub

```bash
# 添加远程仓库（替换为你的GitHub用户名）
git remote add origin https://github.com/YOUR_USERNAME/voltage_modbus.git

# 推送代码
git push -u origin main
```

### 3. 设置GitHub Pages

1. **进入仓库设置**:
   - 在GitHub仓库页面，点击 "Settings" 标签
   
2. **配置Pages**:
   - 在左侧菜单找到 "Pages"
   - Source: 选择 "GitHub Actions"
   - 保存设置

3. **启用GitHub Actions**:
   - 点击 "Actions" 标签
   - 启用workflows

### 4. 配置仓库权限

在仓库设置中：

1. **Actions权限**:
   - Settings → Actions → General
   - Workflow permissions: 选择 "Read and write permissions"
   - 勾选 "Allow GitHub Actions to create and approve pull requests"

2. **Pages权限**:
   - Settings → Pages
   - 确保Source设置为 "GitHub Actions"

### 5. 更新README中的链接

如果你的GitHub用户名不是 `voltage-llc`，需要更新README.md中的链接：

```bash
# 替换所有GitHub链接中的用户名
sed -i 's/voltage-llc/YOUR_USERNAME/g' README.md
sed -i 's/voltage-llc/YOUR_USERNAME/g' .github/workflows/docs.yml
sed -i 's/voltage-llc/YOUR_USERNAME/g' CHANGELOG.md
```

## 📚 文档部署

### 自动部署
- **触发条件**: 每次推送到main分支
- **部署地址**: `https://YOUR_USERNAME.github.io/voltage_modbus/`
- **更新频率**: 实时（推送后约2-5分钟）

### 手动部署
如果需要手动触发文档部署：

```bash
# 触发GitHub Actions
git commit --allow-empty -m "docs: trigger documentation deployment"
git push origin main
```

## 🔧 验证部署

### 1. 检查CI状态
- 访问 `https://github.com/YOUR_USERNAME/voltage_modbus/actions`
- 确保所有workflows都通过✅

### 2. 访问文档
- 主文档: `https://YOUR_USERNAME.github.io/voltage_modbus/`
- API文档: `https://YOUR_USERNAME.github.io/voltage_modbus/voltage_modbus/`

### 3. 测试徽章
README中的徽章应该显示：
- ✅ Build Status: Passing
- 📚 Docs: Deployed

## 🏷️ 创建Release

### 1. 创建标签
```bash
git tag -a v0.1.0 -m "Release version 0.1.0"
git push origin v0.1.0
```

### 2. GitHub Release
1. 访问仓库的 "Releases" 页面
2. 点击 "Create a new release"
3. 选择标签 `v0.1.0`
4. 标题: `Voltage Modbus v0.1.0`
5. 描述: 复制CHANGELOG.md中的内容
6. 点击 "Publish release"

## 📊 监控和维护

### 定期检查
- **GitHub Actions**: 确保CI/CD正常运行
- **Documentation**: 验证文档链接有效
- **Issues**: 及时回复用户问题
- **Dependencies**: 定期更新依赖项

### 性能监控
- 查看GitHub Insights了解项目统计
- 监控文档访问量
- 收集用户反馈

## 🔗 完成后的链接

部署完成后，你的项目将有以下链接：

- **仓库主页**: `https://github.com/YOUR_USERNAME/voltage_modbus`
- **在线文档**: `https://YOUR_USERNAME.github.io/voltage_modbus/`
- **Releases**: `https://github.com/YOUR_USERNAME/voltage_modbus/releases`
- **Issues**: `https://github.com/YOUR_USERNAME/voltage_modbus/issues`
- **Actions**: `https://github.com/YOUR_USERNAME/voltage_modbus/actions`

## 🎉 发布完成！

恭喜！你的Voltage Modbus库现在已经：
- ✅ 在GitHub上公开发布
- ✅ 拥有自动化CI/CD流程
- ✅ 具备在线文档（GitHub Pages）
- ✅ 支持自动更新文档
- ✅ 具备完整的项目结构

---

**联系方式**: Evan Liu <evan.liu@voltageenergy.com>  
**项目许可**: MIT License 